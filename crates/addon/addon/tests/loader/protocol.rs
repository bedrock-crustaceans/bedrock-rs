//! Every `LoaderError` variant: an input that answers nothing the loader asked
//! for is refused and leaves the loader as it was. The `send_static` asserts.

use bedrock_addon::load::{
    DirEntry, EntryKind, Input, Job, JobDone, Loader, LoaderError, Output, PackLoader,
    RepositoryLoader, Request, RootId, Sans, StackLoader,
};
use bedrock_addon::pack::OpenOptions;
use bedrock_addon::storage::{ContentIdentity, PackPath};
use bytes::Bytes;

use super::responder::{Order, Responder, manifest, place};

/// Jobs, their results and inputs cross threads; every loader is `Send + Sync + 'static`.
#[test]
fn jobs_inputs_and_loaders_are_send_and_static() {
    const fn send_static<T: Send + 'static>() {}
    const fn loader<T, L: Loader<T>>() {}
    send_static::<Job>();
    send_static::<JobDone>();
    send_static::<Input>();
    loader::<_, PackLoader>();
    loader::<_, RepositoryLoader>();
    loader::<_, StackLoader>();
}

fn loader() -> (PackLoader, Responder) {
    let mut responder = Responder::new(Order::Issue);
    responder.file(1, "manifest.json", manifest(1, "data"));
    (
        PackLoader::directory(RootId(1), OpenOptions::default()),
        responder,
    )
}

/// The first request, answered.
fn first(loader: &mut PackLoader) -> Request {
    match loader.poll() {
        Some(Output::Request(request)) => request,
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_answer_for_a_place_never_requested_is_unrequested() {
    let (mut loader, mut responder) = loader();
    let request = first(&mut loader);
    let error = loader
        .handle(Input::File {
            location: place(1, "other.json"),
            bytes: Bytes::new(),
        })
        .unwrap_err();
    assert!(
        matches!(error, LoaderError::Unrequested { location } if location == place(1, "other.json"))
    );
    // Nothing changed: the real answer still finishes the load.
    let input = responder.answer_item(request);
    loader.handle(input).unwrap();
    assert!(responder.drive(&mut loader).is_ok());
}

#[test]
fn a_file_for_a_listing_is_a_wrong_answer() {
    let (mut loader, mut responder) = loader();
    let Request::ListDir { location, .. } = first(&mut loader) else {
        panic!("the folder is listed first");
    };
    let error = loader
        .handle(Input::File {
            location: location.clone(),
            bytes: Bytes::new(),
        })
        .unwrap_err();
    assert!(matches!(error, LoaderError::WrongAnswer { .. }));
    loader
        .handle(Input::Dir {
            location,
            entries: vec![DirEntry {
                name: "manifest.json".into(),
                lossy: false,
                kind: EntryKind::File {
                    len: manifest(1, "data").len() as u64,
                },
            }],
        })
        .unwrap();
    // A listing for the manifest read is a wrong answer too.
    let Some(Output::Request(Request::ReadFile { location, .. })) = loader.poll() else {
        panic!("the manifest is read");
    };
    let error = loader
        .handle(Input::Dir {
            location: location.clone(),
            entries: Vec::new(),
        })
        .unwrap_err();
    assert!(matches!(error, LoaderError::WrongAnswer { .. }));
    let input = responder.answer_item(Request::ReadFile {
        location,
        range: bedrock_addon::load::ReadRange::Whole,
        len_hint: None,
    });
    loader.handle(input).unwrap();
    assert!(responder.drive(&mut loader).is_ok());
}

#[test]
fn a_job_fed_back_twice_is_unknown() {
    let (mut loader, responder) = loader();
    let job = loop {
        match loader.poll() {
            Some(Output::Request(request)) => {
                let input = responder.respond(&request);
                loader.handle(input).unwrap();
            }
            Some(Output::Job(job)) => break job,
            other => panic!("{other:?}"),
        }
    };
    let id = job.id();
    loader.handle(Input::JobDone(job.run())).unwrap();
    let error = loader.handle(Input::JobPanicked(id)).unwrap_err();
    assert!(matches!(error, LoaderError::UnknownJob { id: unknown } if unknown == id));
}

#[test]
fn a_key_that_was_not_asked_for_is_unrequested() {
    let (mut loader, _) = loader();
    let error = loader
        .handle(Input::Key {
            identity: ContentIdentity::new("nobody"),
            key: None,
        })
        .unwrap_err();
    assert!(
        matches!(error, LoaderError::UnrequestedKey { identity } if identity.as_str() == "nobody")
    );
}

#[test]
fn input_after_done_is_finished() {
    let (mut loader, mut responder) = loader();
    let pack = responder.drive(&mut loader).unwrap();
    assert_eq!(
        pack.paths().map(PackPath::as_str).collect::<Vec<_>>(),
        ["manifest.json"]
    );
    assert!(loader.poll().is_none());
    let error = loader
        .handle(Input::Missing {
            location: place(1, ""),
        })
        .unwrap_err();
    assert!(matches!(error, LoaderError::Finished));
}
