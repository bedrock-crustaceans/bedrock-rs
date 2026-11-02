//! The bookkeeping every loader shares: what it asked and has not been answered, the read and
//! job windows, the key requests and the progress counters. Nothing here parses or reads;
//! `handle` and `poll` stay cheap.

use std::collections::{HashMap, HashSet, VecDeque};

use bedrock_addon_core::load::{Limits, ListDepth, Place, ReadRange};

use super::job::{Cancel, Job, JobId, JobKind, Work};
use super::{KeyRequest, LoaderError, Output, Progress, Request};
use crate::storage::ContentIdentity;

/// An output other than `Done`.
#[derive(Debug)]
pub(crate) enum Ask {
    Request(Request),
    NeedKey(KeyRequest),
    Job(Job),
}

impl Ask {
    /// As a loader output.
    pub(crate) fn into_output<T>(self) -> Output<T> {
        match self {
            Ask::Request(request) => Output::Request(request),
            Ask::NeedKey(request) => Output::NeedKey(request),
            Ask::Job(job) => Output::Job(job),
        }
    }
}

/// Whether a read's charge outlives its answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Charge {
    /// The charge is released with the answer.
    Release,
    /// The charge stays until [`Io::release`] (a loose file parsed under `Transient`).
    Hold,
}

/// Which kind of request an answer must be for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expect {
    /// A `ListDir` answer.
    List,
    /// A `ReadFile` answer.
    Read,
    /// Either kind.
    Any,
}

/// An issued request without an answer yet.
#[derive(Clone, Copy, Debug)]
enum Pending {
    List,
    Read {
        /// What the read is charged against the byte window.
        charge: u64,
        /// Whether the charge outlives the answer.
        hold: Charge,
    },
}

/// What an answer answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Answered {
    /// A `ListDir`.
    List,
    /// A `ReadFile`.
    Read,
}

/// A request waiting for its place to be answered.
#[derive(Debug)]
enum Deferred {
    List(ListDepth),
    Read {
        range: ReadRange,
        len_hint: Option<u64>,
        hold: Charge,
    },
}

/// A read waiting for the window.
#[derive(Debug)]
struct WaitingRead {
    location: Place,
    range: ReadRange,
    len_hint: Option<u64>,
    charge: u64,
    hold: Charge,
}

/// One loader's requests, keys, jobs, windows and counters.
#[derive(Debug)]
pub(crate) struct Io {
    limits: Limits,
    ready: VecDeque<Ask>,
    waiting_reads: VecDeque<WaitingRead>,
    /// The places of `waiting_reads`, each there once: what `busy` asks, so that queueing a
    /// read does not scan the queue.
    queued: HashSet<Place>,
    waiting_jobs: VecDeque<Job>,
    outstanding: HashMap<Place, Pending>,
    /// Requests for a place that has one outstanding or queued: issued, in order, once it is
    /// answered: never two requests for one location at once.
    deferred: HashMap<Place, VecDeque<Deferred>>,
    /// Charges kept after their read was answered, released by [`Io::release`].
    held: HashMap<Place, u64>,
    reads_out: u32,
    charged: u64,
    jobs_out: HashMap<JobId, JobKind>,
    next_job: u64,
    keys_asked: HashSet<ContentIdentity>,
    keys_pending: HashSet<ContentIdentity>,
    cancel: Cancel,
    pub(crate) progress: Progress,
}

impl Io {
    pub(crate) fn new(limits: Limits) -> Io {
        Io {
            limits,
            ready: VecDeque::new(),
            waiting_reads: VecDeque::new(),
            queued: HashSet::new(),
            waiting_jobs: VecDeque::new(),
            outstanding: HashMap::new(),
            deferred: HashMap::new(),
            held: HashMap::new(),
            reads_out: 0,
            charged: 0,
            jobs_out: HashMap::new(),
            next_job: 0,
            keys_asked: HashSet::new(),
            keys_pending: HashSet::new(),
            cancel: Cancel::default(),
            progress: Progress::default(),
        }
    }

    /// Asks for a listing; listings are not windowed.
    pub(crate) fn list(&mut self, location: Place, depth: ListDepth) {
        if self.busy(&location) {
            self.deferred
                .entry(location)
                .or_default()
                .push_back(Deferred::List(depth));
            return;
        }
        self.issue_list(location, depth);
    }

    /// Issues a listing now.
    fn issue_list(&mut self, location: Place, depth: ListDepth) {
        self.outstanding.insert(location.clone(), Pending::List);
        self.progress.requests_issued += 1;
        self.ready
            .push_back(Ask::Request(Request::ListDir { location, depth }));
    }

    /// Queues a read. The charge is `n` for `Prefix(n)`, else the size hint, else the archive
    /// limit; with [`Charge::Hold`] it stays charged after the answer until [`Io::release`].
    pub(crate) fn read(
        &mut self,
        location: Place,
        range: ReadRange,
        len_hint: Option<u64>,
        hold: Charge,
    ) {
        if self.busy(&location) {
            self.deferred
                .entry(location)
                .or_default()
                .push_back(Deferred::Read {
                    range,
                    len_hint,
                    hold,
                });
            return;
        }
        self.queue_read(location, range, len_hint, hold);
    }

    /// Puts a read in the window's queue.
    fn queue_read(
        &mut self,
        location: Place,
        range: ReadRange,
        len_hint: Option<u64>,
        hold: Charge,
    ) {
        let charge = match range {
            ReadRange::Prefix(n) => u64::from(n),
            ReadRange::Whole => len_hint.unwrap_or(self.limits.max_archive_bytes),
        };
        self.queued.insert(location.clone());
        self.waiting_reads.push_back(WaitingRead {
            location,
            range,
            len_hint,
            charge,
            hold,
        });
        self.pump();
    }

    /// Asks the caller for a content key, once per identity per loader; `false` when it was
    /// asked already (the answer serves every pack waiting for it).
    pub(crate) fn ask_key(&mut self, request: KeyRequest) -> bool {
        if !self.keys_asked.insert(request.identity.clone()) {
            return false;
        }
        self.keys_pending.insert(request.identity.clone());
        self.ready.push_back(Ask::NeedKey(request));
        true
    }

    /// Queues a job and returns its id.
    pub(crate) fn job(&mut self, kind: JobKind, weight: u64, work: Work) -> JobId {
        let id = JobId(self.next_job);
        self.next_job += 1;
        self.waiting_jobs
            .push_back(Job::new(id, kind, weight, self.cancel.clone(), work));
        self.pump();
        id
    }

    /// Issues what the windows allow, in queue order.
    fn pump(&mut self) {
        while let Some(read) = self.waiting_reads.front() {
            let count_open = self.reads_out < self.limits.max_pending_reads;
            // A read larger than the whole window goes alone.
            let bytes_open = self.charged == 0
                || self.charged.saturating_add(read.charge) <= self.limits.max_pending_read_bytes;
            if !(count_open && bytes_open) {
                break;
            }
            let Some(read) = self.waiting_reads.pop_front() else {
                break;
            };
            self.queued.remove(&read.location);
            self.reads_out += 1;
            self.charged = self.charged.saturating_add(read.charge);
            self.outstanding.insert(
                read.location.clone(),
                Pending::Read {
                    charge: read.charge,
                    hold: read.hold,
                },
            );
            self.progress.requests_issued += 1;
            self.ready.push_back(Ask::Request(Request::ReadFile {
                location: read.location,
                range: read.range,
                len_hint: read.len_hint,
            }));
        }
        while self.jobs_out.len() < self.limits.max_pending_jobs as usize {
            let Some(job) = self.waiting_jobs.pop_front() else {
                break;
            };
            self.jobs_out.insert(job.id(), job.kind());
            self.progress.jobs_issued += 1;
            self.ready.push_back(Ask::Job(job));
        }
    }

    /// Checks that `location` has an outstanding request that an answer of kind `expect`
    /// answers, without changing anything.
    pub(crate) fn check_answer(&self, location: &Place, expect: Expect) -> Result<(), LoaderError> {
        match (self.outstanding.get(location), expect) {
            (None, _) => Err(LoaderError::Unrequested {
                location: location.clone(),
            }),
            (Some(Pending::List), Expect::Read) | (Some(Pending::Read { .. }), Expect::List) => {
                Err(LoaderError::WrongAnswer {
                    location: location.clone(),
                })
            }
            _ => Ok(()),
        }
    }

    /// Records the answer to the request at `location` (checked with [`Io::check_answer`]).
    pub(crate) fn answer(&mut self, location: &Place, bytes: u64) -> Answered {
        self.progress.requests_answered += 1;
        self.progress.bytes_received += bytes;
        let answered = match self.outstanding.remove(location) {
            Some(Pending::Read { charge, hold }) => {
                self.reads_out -= 1;
                if hold == Charge::Hold {
                    self.held.insert(location.clone(), charge);
                } else {
                    self.charged = self.charged.saturating_sub(charge);
                }
                Answered::Read
            }
            Some(Pending::List) | None => Answered::List,
        };
        let next = self
            .deferred
            .get_mut(location)
            .and_then(VecDeque::pop_front);
        if self.deferred.get(location).is_some_and(VecDeque::is_empty) {
            self.deferred.remove(location);
        }
        match next {
            Some(Deferred::List(depth)) => self.issue_list(location.clone(), depth),
            Some(Deferred::Read {
                range,
                len_hint,
                hold,
            }) => self.queue_read(location.clone(), range, len_hint, hold),
            None => {}
        }
        self.pump();
        answered
    }

    /// A request for `location` is outstanding, queued or deferred.
    fn busy(&self, location: &Place) -> bool {
        self.outstanding.contains_key(location)
            || self.deferred.contains_key(location)
            || self.queued.contains(location)
    }

    /// The outstanding request at `location` is a listing.
    pub(crate) fn is_listing(&self, location: &Place) -> bool {
        matches!(self.outstanding.get(location), Some(Pending::List))
    }

    /// Releases the charge a held read keeps.
    pub(crate) fn release(&mut self, location: &Place) {
        if let Some(charge) = self.held.remove(location) {
            self.charged = self.charged.saturating_sub(charge);
            self.pump();
        }
    }

    /// Checks that a key for `identity` was asked for and not answered yet.
    pub(crate) fn check_key(&self, identity: &ContentIdentity) -> Result<(), LoaderError> {
        if self.keys_pending.contains(identity) {
            Ok(())
        } else {
            Err(LoaderError::UnrequestedKey {
                identity: identity.clone(),
            })
        }
    }

    /// Records the answer for `identity` (checked with [`Io::check_key`]).
    pub(crate) fn answer_key(&mut self, identity: &ContentIdentity) {
        self.keys_pending.remove(identity);
    }

    /// Checks that job `id` is outstanding.
    pub(crate) fn check_job(&self, id: JobId) -> Result<JobKind, LoaderError> {
        self.jobs_out
            .get(&id)
            .copied()
            .ok_or(LoaderError::UnknownJob { id })
    }

    /// Records job `id` fed back (checked with [`Io::check_job`]).
    pub(crate) fn job_done(&mut self, id: JobId) {
        if self.jobs_out.remove(&id).is_some() {
            self.progress.jobs_done += 1;
        }
        self.pump();
    }

    /// The next output to hand out.
    pub(crate) fn poll(&mut self) -> Option<Ask> {
        self.ready.pop_front()
    }

    /// Nothing is ready, queued or outstanding: `Done` may follow.
    pub(crate) fn is_idle(&self) -> bool {
        self.ready.is_empty()
            && self.waiting_reads.is_empty()
            && self.waiting_jobs.is_empty()
            && self.outstanding.is_empty()
            && self.deferred.is_empty()
            && self.jobs_out.is_empty()
            && self.keys_pending.is_empty()
    }

    /// The window limits.
    pub(crate) fn limits(&self) -> &Limits {
        &self.limits
    }
}

/// Dropping the loader cancels its jobs.
impl Drop for Io {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::RootId;
    use bedrock_addon_core::pack_identity::PackPath;

    use super::*;

    fn places(count: usize) -> Vec<Place> {
        (0..count)
            .map(|n| {
                let path = PackPath::new(&format!("entity/e{n:06}.json")).unwrap();
                Place::root(RootId(1)).join(&path)
            })
            .collect()
    }

    /// Queues a read of each of `places`: what a stack load does at its start for every loose
    /// file a collection walks (the window lets 64 out; the rest wait).
    fn queue(places: Vec<Place>) {
        let mut io = Io::new(Limits::default());
        for place in places {
            io.read(place, ReadRange::Whole, Some(1), Charge::Hold);
        }
        assert_eq!(io.reads_out, 64);
    }

    // Queueing the reads of a stack load must cost time linear in the reads waiting. A queued
    // read once scanned every read still waiting for the window, to keep two requests for one
    // place apart, so a load of one pack grew by a factor of two to three per doubling of its
    // files. The queue now keeps the waiting places in a set, and this test pins the linear
    // cost.
    #[test]
    fn queueing_reads_is_linear_in_the_reads_waiting() {
        crate::test_cost::assert_linear_consuming_at(
            "queueing a read of every loose file",
            20_000,
            places,
            queue,
        );
    }

    fn asked(io: &mut Io) -> Vec<String> {
        std::iter::from_fn(|| io.poll())
            .map(|ask| match ask {
                Ask::Request(Request::ReadFile { location, .. }) => location.path.to_string(),
                other => format!("{other:?}"),
            })
            .collect()
    }

    // Never two requests for one location at once, whether the first is out
    // or still waiting for the window; the second is issued once the first is answered.
    #[test]
    fn a_second_read_of_a_place_waits_for_the_first_out_or_queued() {
        let limits = Limits {
            max_pending_reads: 1,
            ..Limits::default()
        };
        let mut io = Io::new(limits);
        let [a, b] = [0, 1].map(|n| places(2).swap_remove(n));
        io.read(a.clone(), ReadRange::Whole, Some(1), Charge::Release);
        // `b` waits for the window; a second read of it waits for the first.
        io.read(b.clone(), ReadRange::Whole, Some(1), Charge::Release);
        io.read(b.clone(), ReadRange::Whole, Some(1), Charge::Release);
        // And a second read of `a`, which is out.
        io.read(a.clone(), ReadRange::Whole, Some(1), Charge::Release);
        assert_eq!(asked(&mut io), ["entity/e000000.json"]);
        io.answer(&a, 1);
        // The window is one read wide: `b`, queued before the second `a`.
        assert_eq!(asked(&mut io), ["entity/e000001.json"]);
        io.answer(&b, 1);
        assert_eq!(asked(&mut io), ["entity/e000000.json"]);
        io.answer(&a, 1);
        assert_eq!(asked(&mut io), ["entity/e000001.json"]);
        io.answer(&b, 1);
        assert_eq!(asked(&mut io), [] as [&str; 0]);
        assert!(io.is_idle());
    }
}
