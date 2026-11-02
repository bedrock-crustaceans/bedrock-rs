//! Unit tests of the version rules and the resolver.

use super::resolve::{
    Entry, Request, VersionResolution, conflicts, pair_conflicts, requested, resolve, table,
};
use super::{ScriptRuntime, ScriptVersion};
use bedrock_addon_core::manifest::NativeModuleVersion;
use bedrock_addon_core::version::SemVersion;

fn v(text: &str) -> ScriptVersion {
    ScriptVersion::parse(text).unwrap_or_else(|| panic!("{text} is no version"))
}

/// What the dedicated server serves of `@minecraft/server`, the hidden internal version included.
fn server_versions() -> Vec<ScriptVersion> {
    [
        "0.1.0",
        "1.19.0",
        "2.8.0",
        "2.9.0-beta",
        "2.9.0-internal",
        "3.0.0-alpha",
    ]
    .map(v)
    .to_vec()
}

fn own(version: &str) -> Request {
    Request {
        version: v(version),
        by: "pack - 3.2.1".to_owned(),
        explicit: true,
    }
}

fn transitive(version: &str) -> Request {
    Request {
        version: v(version),
        by: "@minecraft/server-ui-bindings - 2.1.0".to_owned(),
        explicit: false,
    }
}

// ---- ScriptVersion ----

#[test]
fn versions_order_as_semver_precedence() {
    let chain = [
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
    ]
    .map(v);
    for (i, low) in chain.iter().enumerate() {
        for high in &chain[i + 1..] {
            assert!(low < high, "{low} must be below {high}");
            assert!(high > low, "{high} must be above {low}");
        }
        assert_eq!(low.cmp(low), std::cmp::Ordering::Equal);
    }
}

#[test]
fn a_pre_release_is_between_the_releases_around_it() {
    assert!(v("2.8.0") < v("2.9.0-beta"));
    assert!(v("2.9.0-beta") < v("2.9.0"));
    assert!(v("2.9.0-beta") < v("2.9.0-internal"));
    assert!(v("2.9.0-internal") < v("3.0.0-alpha"));
}

#[test]
fn numeric_identifiers_compare_by_value_and_sort_below_others_deviation() {
    // Two all-digit identifiers by value, however long.
    assert!(v("1.0.0-rc.2") < v("1.0.0-rc.10"));
    assert!(v("1.0.0-rc.99999999999999999998") < v("1.0.0-rc.99999999999999999999"));
    // An all-digit identifier is below one with any other byte.
    assert!(v("1.0.0-rc.999") < v("1.0.0-rc.a"));
    // Bytewise otherwise, the shorter first when one is a prefix of the other.
    assert!(v("1.0.0-be") < v("1.0.0-beta"));
    assert!(v("1.0.0-Beta") < v("1.0.0-beta"));
}

/// A module version of a manifest is read as a strict semantic version
/// (the relaxed reader of format 3 and the legacy reader of formats 0 to 2 both end in it),
/// whose pattern refuses a numeric pre-release identifier with a leading zero, so `1.0.0-01`
/// is no version. How `01` and `1` would order is moot: no version can spell them apart.
#[test]
fn a_numeric_pre_release_identifier_with_a_leading_zero_is_no_version() {
    assert_eq!(ScriptVersion::parse("1.0.0-01"), None);
    assert_eq!(ScriptVersion::parse("1.0.0-1"), Some(v("1.0.0-1")));
    assert!(ScriptVersion::parse("1.0.0-0a").is_some());
}

#[test]
fn a_version_drops_build_metadata_and_prints_as_vanilla_does() {
    assert_eq!(v("2.0.0+abc"), ScriptVersion::new(2, 0, 0));
    assert_eq!(v("2.9.0-beta").to_string(), "2.9.0-beta");
    assert_eq!(ScriptVersion::new(3, 2, 1).to_string(), "3.2.1");
    let sem = SemVersion::parse("2.0.0-beta.1+build").expect("a version");
    assert_eq!(
        ScriptVersion::from(&sem),
        ScriptVersion::with_pre(2, 0, 0, "beta.1")
    );
    for text in ["2.0", "2", "*", "abc", ""] {
        assert_eq!(ScriptVersion::parse(text), None, "{text:?}");
    }
}

/// The truth table: five candidates against ten requests, both modes.
#[test]
fn satisfies_truth_table() {
    let candidates = ["0.1.0", "1.19.0", "2.8.0", "2.9.0-beta", "3.0.0-alpha"];
    let requests = [
        "0.1.0",
        "0.2.0",
        "1.0.0",
        "1.19.0",
        "1.20.0",
        "2.8.0",
        "2.9.0-beta",
        "2.8.0-beta",
        "3.0.0-alpha",
        "2.0.0-rc",
    ];
    let strict = [
        ("0.1.0", "0.1.0"),
        ("1.19.0", "1.0.0"),
        ("1.19.0", "1.19.0"),
        ("2.8.0", "2.8.0"),
        ("2.9.0-beta", "2.9.0-beta"),
        ("3.0.0-alpha", "3.0.0-alpha"),
    ];
    // Without `strict`, a candidate with a tag also serves a release request of its major.
    let lenient_only = [("2.9.0-beta", "2.8.0")];
    for candidate in candidates {
        for request in requests {
            let pair = (candidate, request);
            let in_strict = strict.contains(&pair);
            assert_eq!(
                v(candidate).satisfies(&v(request), true),
                in_strict,
                "strict: candidate {candidate}, request {request}"
            );
            assert_eq!(
                v(candidate).satisfies(&v(request), false),
                in_strict || lenient_only.contains(&pair),
                "lenient: candidate {candidate}, request {request}"
            );
        }
    }
}

#[test]
fn the_beta_gate_is_the_beta_tag_or_major_zero() {
    assert!(v("2.9.0-beta").is_beta());
    assert!(v("0.1.0").is_beta());
    assert!(v("0.0.0-alpha").is_beta());
    assert!(!v("2.8.0").is_beta());
    assert!(!v("2.0.0-beta.1").is_beta());
    assert!(!v("3.0.0-alpha").is_beta());
    assert!(v("2.9.0-internal").is_internal());
    assert!(!v("2.9.0-internal.1").is_internal());
    assert!(!v("2.9.0").is_internal());
}

// ---- ScriptRuntime ----

#[test]
fn runtime_of_a_language() {
    let js = Some(ScriptRuntime::JavaScript);
    assert_eq!(ScriptRuntime::from_language(None), js);
    assert_eq!(ScriptRuntime::from_language(Some("")), js);
    assert_eq!(ScriptRuntime::from_language(Some("javascript")), js);
    assert_eq!(ScriptRuntime::from_language(Some("jAvAsCrIpT")), js);
    assert_eq!(ScriptRuntime::from_language(Some("JAVASCRIPT")), js);
    for other in [
        "javascript ",
        " javascript",
        "js",
        "typescript",
        "java script",
        "javascrip",
    ] {
        assert_eq!(ScriptRuntime::from_language(Some(other)), None, "{other:?}");
    }
    assert_eq!(ScriptRuntime::JavaScript.name(), "JavaScript");
    assert_eq!(ScriptRuntime::JavaScript.extension(), ".js");
}

// ---- requested ----

#[test]
fn a_selector_asks_for_the_newest_version_of_its_channel() {
    let supported = server_versions();
    assert_eq!(
        requested(&NativeModuleVersion::LatestBeta, &supported),
        v("2.9.0-beta")
    );
    assert_eq!(
        requested(&NativeModuleVersion::LatestInternal, &supported),
        v("2.9.0-internal")
    );
    assert_eq!(
        requested(&NativeModuleVersion::LatestAlpha, &supported),
        v("3.0.0-alpha")
    );
    let two_betas = ["2.9.0-beta", "2.10.0-beta", "2.2.0-beta"].map(v);
    assert_eq!(
        requested(&NativeModuleVersion::LatestBeta, &two_betas),
        v("2.10.0-beta")
    );
}

#[test]
fn a_selector_without_a_version_of_its_channel_asks_for_zero() {
    let releases = ["1.19.0", "2.8.0"].map(v);
    assert_eq!(
        requested(&NativeModuleVersion::LatestBeta, &releases),
        v("0.0.0-beta")
    );
    assert_eq!(
        requested(&NativeModuleVersion::LatestAlpha, &[]),
        v("0.0.0-alpha")
    );
    assert_eq!(
        requested(&NativeModuleVersion::LatestInternal, &[]),
        v("0.0.0-internal")
    );
    // Which the Beta APIs gate then covers, by its major.
    assert!(requested(&NativeModuleVersion::LatestBeta, &releases).is_beta());
}

#[test]
fn a_version_asks_for_itself_without_build_metadata() {
    let sem = SemVersion::parse("2.0.0+abc").expect("a version");
    assert_eq!(
        requested(&NativeModuleVersion::Version(sem), &server_versions()),
        v("2.0.0")
    );
}

// ---- resolve ----

fn resolved(requests: &[Request]) -> (ScriptVersion, Vec<ScriptVersion>) {
    match resolve(requests, &server_versions()) {
        VersionResolution::Resolved {
            version, promoted, ..
        } => (version, promoted.into_iter().map(|e| e.version).collect()),
        failed @ VersionResolution::Failed { .. } => panic!("expected a version, got {failed:?}"),
    }
}

fn failed(requests: &[Request]) -> (Vec<(ScriptVersion, ScriptVersion)>, Vec<ScriptVersion>) {
    match resolve(requests, &server_versions()) {
        VersionResolution::Failed { conflicts, missing } => (
            conflicts
                .into_iter()
                .map(|(a, b)| (a.version, b.version))
                .collect(),
            missing.into_iter().map(|e| e.version).collect(),
        ),
        other @ VersionResolution::Resolved { .. } => panic!("expected a failure, got {other:?}"),
    }
}

#[test]
fn a_release_request_is_served_by_the_newest_release_of_its_major() {
    assert_eq!(resolved(&[own("2.0.0")]), (v("2.8.0"), vec![]));
    assert_eq!(resolved(&[own("1.5.1")]), (v("1.19.0"), vec![]));
    assert_eq!(resolved(&[own("2.8.0")]), (v("2.8.0"), vec![]));
    assert_eq!(resolved(&[own("0.1.0")]), (v("0.1.0"), vec![]));
    assert_eq!(resolved(&[own("2.9.0-beta")]), (v("2.9.0-beta"), vec![]));
}

#[test]
fn a_request_no_version_serves_is_missing() {
    for version in [
        "9.0.0",
        "2.99.0",
        "1.0.0-rc",
        "2.0.0-beta.1",
        "0.2.0",
        "2.8.0-beta",
    ] {
        assert_eq!(
            failed(&[own(version)]),
            (vec![], vec![v(version)]),
            "{version}"
        );
    }
}

#[test]
fn requests_in_one_major_merge() {
    assert_eq!(
        resolved(&[own("2.0.0"), own("2.8.0")]),
        (v("2.8.0"), vec![])
    );
    assert_eq!(
        resolved(&[own("2.0.0"), own("2.0.0")]),
        (v("2.8.0"), vec![])
    );
    assert_eq!(
        resolved(&[own("1.0.0"), transitive("1.3.0")]),
        (v("1.19.0"), vec![])
    );
}

#[test]
fn requests_in_two_majors_conflict_and_a_served_request_is_not_missing() {
    // Neither 1.5.0 nor 2.0.0 is printed as an invalid version, only the conflict.
    assert_eq!(
        failed(&[own("1.5.0"), own("2.0.0")]),
        (vec![(v("1.5.0"), v("2.0.0"))], vec![])
    );
}

#[test]
fn a_conflict_also_reports_the_requests_nothing_serves() {
    // 9.0.0 is served by nothing, and is reported besides the conflict.
    assert_eq!(
        failed(&[own("1.5.0"), own("9.0.0")]),
        (vec![(v("1.5.0"), v("9.0.0"))], vec![v("9.0.0")])
    );
}

#[test]
fn the_plugins_own_pre_release_serves_a_release_request_in_either_order() {
    // The request for 2.0.0 is promoted because of the other request.
    assert_eq!(
        resolved(&[own("2.0.0"), own("2.9.0-beta")]),
        (v("2.9.0-beta"), vec![v("2.0.0")])
    );
    assert_eq!(
        resolved(&[own("2.9.0-beta"), own("2.0.0")]),
        (v("2.9.0-beta"), vec![v("2.0.0")])
    );
    // So does a module's release request.
    assert_eq!(
        resolved(&[own("2.9.0-beta"), transitive("2.0.0")]),
        (v("2.9.0-beta"), vec![v("2.0.0")])
    );
}

#[test]
fn a_modules_pre_release_request_does_not_serve_the_plugins_lower_release() {
    // In one major: the higher request is a module's and the plugin's first is lower.
    assert_eq!(
        failed(&[own("2.0.0"), transitive("2.9.0-beta")]),
        (vec![(v("2.0.0"), v("2.9.0-beta"))], vec![])
    );
    // No request of the plugin at all: strict as well.
    assert_eq!(
        failed(&[transitive("2.0.0"), transitive("2.9.0-beta")]),
        (vec![(v("2.0.0"), v("2.9.0-beta"))], vec![])
    );
    // The plugin's first request is not lower than the module's: not strict.
    assert_eq!(
        resolved(&[
            own("2.9.0-beta"),
            transitive("2.9.0-beta"),
            transitive("2.0.0")
        ])
        .0,
        v("2.9.0-beta")
    );
}

#[test]
fn a_transitive_request_in_another_major_conflicts() {
    // A module's request for another major conflicts with the plugin's own.
    let (pairs, missing) = failed(&[own("1.19.0"), transitive("2.0.0")]);
    assert_eq!(pairs, vec![(v("1.19.0"), v("2.0.0"))]);
    assert_eq!(missing, vec![]);
}

#[test]
fn the_table_puts_the_promotion_before_its_request() {
    let rows = table(
        &[own("2.0.0"), own("9.0.0"), own("2.8.0")],
        &server_versions(),
    );
    let shape: Vec<(String, bool, bool, bool)> = rows
        .iter()
        .map(|e| (e.version.to_string(), e.promotion, e.supported, e.missing))
        .collect();
    assert_eq!(
        shape,
        vec![
            ("2.8.0".to_owned(), true, true, false),
            ("2.0.0".to_owned(), false, false, false),
            ("9.0.0".to_owned(), false, false, true),
            ("2.8.0".to_owned(), false, true, false),
        ]
    );
    assert!(rows.iter().all(|e| e.explicit));
}

#[test]
fn equal_requests_are_one_row() {
    let rows = table(
        &[
            own("2.0.0"),
            own("2.0.0"),
            transitive("2.0.0"),
            transitive("2.0.0"),
        ],
        &server_versions(),
    );
    let requests: Vec<bool> = rows
        .iter()
        .filter(|e| !e.promotion)
        .map(|e| e.explicit)
        .collect();
    assert_eq!(requests, vec![true, false]);
}

// ---- the conflict search against the pairwise rule ----

/// A small deterministic generator (no dependency on a random crate).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % bound
    }
}

fn row(version: ScriptVersion, explicit: bool) -> Entry {
    Entry {
        version,
        by: String::new(),
        explicit,
        supported: false,
        missing: false,
        promotion: false,
    }
}

/// The later rows that conflict with an earlier request row, pair by pair.
fn conflicting_rows_pairwise(rows: &[Entry]) -> Vec<usize> {
    let first_explicit = rows.iter().find(|e| e.explicit).map(|e| e.version.clone());
    (0..rows.len())
        .filter(|&b| !rows[b].promotion)
        .filter(|&b| {
            (0..b).any(|a| {
                !rows[a].promotion && pair_conflicts(&rows[a], &rows[b], first_explicit.as_ref())
            })
        })
        .collect()
}

#[test]
fn the_pairwise_rule_is_vanillas() {
    let first = v("1.19.0");
    let pair = |a: &str, b: &str, b_explicit: bool| {
        pair_conflicts(&row(v(a), true), &row(v(b), b_explicit), Some(&first))
    };
    assert!(pair("1.19.0", "2.0.0", true), "two majors");
    assert!(
        pair("2.0.0", "1.19.0", true),
        "two majors, the higher first"
    );
    assert!(!pair("2.0.0", "2.8.0", true), "one major, releases");
    assert!(
        !pair("2.0.0", "2.9.0-beta", true),
        "the plugin's own pre-release"
    );
    assert!(
        pair("2.0.0", "2.9.0-beta", false),
        "a module's pre-release, strict"
    );
    assert!(
        pair("2.9.0-beta", "2.8.0-beta", true),
        "two versions of one tag"
    );
    assert!(pair("2.9.0-beta", "2.9.0-internal", true), "two tags");
    assert!(
        pair("2.9.0", "2.8.0-beta", true),
        "a release above a pre-release"
    );
    assert!(pair("0.1.0", "0.2.0", true), "major 0 is exact");
    assert!(!pair("0.1.0", "0.1.0", true), "equal versions");
    assert!(
        !pair("2.9.0-beta", "2.9.0-beta", false),
        "equal pre-releases"
    );
}

/// Two implementations agree: the linear conflict search finds a partner for exactly the
/// rows the pairwise rule finds one for, and every partner it names does conflict.
#[test]
fn the_conflict_search_agrees_with_the_pairwise_rule() {
    let tags = ["", "", "", "beta", "alpha", "beta.1"];
    let mut rng = Lcg(0x5eed);
    let (mut rounds_with_conflicts, mut rounds_without) = (0, 0);
    for round in 0..4000 {
        let len = 1 + usize::try_from(rng.next(7)).unwrap();
        let majors = 1 + rng.next(3);
        let mut rows = Vec::new();
        for _ in 0..len {
            let version = ScriptVersion::with_pre(
                u16::try_from(rng.next(majors) + u64::from(round % 2 == 0)).unwrap(),
                u16::try_from(rng.next(3)).unwrap(),
                0,
                tags[usize::try_from(rng.next(6)).unwrap()],
            );
            let mut entry = row(version, rng.next(2) == 0);
            entry.promotion = rng.next(5) == 0;
            // A table holds one row per version and asker kind.
            if !rows
                .iter()
                .any(|e: &Entry| e.version == entry.version && e.explicit == entry.explicit)
            {
                rows.push(entry);
            }
        }
        let first_explicit = rows.iter().find(|e| e.explicit).map(|e| e.version.clone());
        let found = conflicts(&rows);
        for &(a, b) in &found {
            assert!(a < b, "round {round}: the partner comes first");
            assert!(
                !rows[a].promotion && !rows[b].promotion,
                "round {round}: promotion rows never conflict"
            );
            assert!(
                pair_conflicts(&rows[a], &rows[b], first_explicit.as_ref()),
                "round {round}: rows {a} and {b} of {rows:?} do not conflict"
            );
        }
        rounds_with_conflicts += usize::from(!found.is_empty());
        rounds_without += usize::from(found.is_empty() && rows.len() > 1);
        let later: Vec<usize> = found.iter().map(|&(_, b)| b).collect();
        assert_eq!(
            later,
            conflicting_rows_pairwise(&rows),
            "round {round}: {rows:?}"
        );
    }
    // Both outcomes occur often enough to mean something.
    assert!(
        rounds_with_conflicts > 300,
        "{rounds_with_conflicts} rounds with a conflict"
    );
    assert!(
        rounds_without > 300,
        "{rounds_without} rounds of several rows without one"
    );
}

// ---- cost: linear in the requests, the dependencies and the catalog's chains ----

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::pack_identity::PackPath;
use uuid::Uuid;

use super::{
    CatalogModule, CatalogVersion, ModuleCatalog, NativeDependency, ScriptContext, ScriptDiscovery,
    ScriptFiles, ScriptModule, ScriptTarget,
};
use bedrock_addon_core::manifest::dependency_texts;
use bedrock_addon_core::test_cost;

/// The `i`-th of many distinct releases of major `major`.
fn distinct(major: u16, i: usize) -> ScriptVersion {
    ScriptVersion::new(
        major,
        u16::try_from(i / 1000).unwrap(),
        u16::try_from(i % 1000).unwrap(),
    )
}

fn many_requests(n: usize, major_of: impl Fn(usize) -> u16) -> Vec<Request> {
    (0..n)
        .map(|i| Request {
            version: distinct(major_of(i), i),
            by: "pack - 3.2.1".to_owned(),
            explicit: i % 2 == 0,
        })
        .collect()
}

#[test]
fn resolving_many_requests_of_one_major_is_linear() {
    // The newest release of the catalog serves every request: no conflict, one fold.
    let supported = [ScriptVersion::new(1, u16::MAX, 0)];
    test_cost::assert_linear_at(
        "resolve, n requests of one major",
        20_000,
        |n| many_requests(n, |_| 1),
        |requests| {
            let resolution = resolve(requests, &supported);
            assert!(matches!(resolution, VersionResolution::Resolved { .. }));
        },
    );
}

#[test]
fn resolving_many_conflicting_requests_is_linear() {
    // Two majors in turn: every row but the first conflicts with an earlier one, and each is
    // reported once, not once per earlier row.
    test_cost::assert_linear_at(
        "resolve, n requests in two majors",
        20_000,
        |n| many_requests(n, |i| if i % 2 == 0 { 1 } else { 2 }),
        |requests| match resolve(requests, &[]) {
            VersionResolution::Failed { conflicts, missing } => {
                assert_eq!(conflicts.len(), requests.len() - 1);
                assert_eq!(missing.len(), requests.len());
            }
            other @ VersionResolution::Resolved { .. } => panic!("{other:?}"),
        },
    );
}

#[test]
fn resolving_many_tagged_requests_is_linear() {
    // One major, releases and pre-releases mixed: the tagged rows conflict among themselves.
    test_cost::assert_linear_at(
        "resolve, n tagged and untagged requests",
        20_000,
        |n| {
            let mut requests = many_requests(n, |_| 2);
            for (i, request) in requests.iter_mut().enumerate() {
                if i % 3 == 0 {
                    request.version.pre = "beta".to_owned();
                }
            }
            requests
        },
        |requests| {
            assert!(matches!(
                resolve(requests, &[]),
                VersionResolution::Failed { .. }
            ));
        },
    );
}

fn script_module(dependencies: Vec<NativeDependency>) -> ScriptModule {
    let mut path = JsonPath::new();
    path.push("modules");
    path.push_index(0);
    ScriptModule {
        pack_name: "pack".to_owned(),
        pack_uuid: Uuid::from_u128(9),
        file: PackPath::from_static("manifest.json"),
        target: ScriptTarget::Server,
        module_uuid: Uuid::from_u128(1),
        module_version: SemVersion::new(3, 2, 1),
        language: None,
        entry: Some("scripts/main.js".to_owned()),
        path,
        skipped: Vec::new(),
        dependencies,
        uuid_dependencies: Vec::new(),
        script_eval: false,
        min_engine_version: None,
    }
}

fn dependency_on(module: &str, version: ScriptVersion, index: usize) -> NativeDependency {
    let mut path = JsonPath::new();
    path.push("dependencies");
    path.push_index(index);
    let mut sem = SemVersion::new(version.major, version.minor, version.patch);
    sem.pre_release = version.pre;
    NativeDependency {
        module_name: module.to_owned(),
        version: NativeModuleVersion::Version(sem),
        optional: false,
        path,
    }
}

fn one_module_catalog(name: &str, versions: &[ScriptVersion]) -> ModuleCatalog {
    let mut catalog = ModuleCatalog::new();
    catalog.insert(
        name,
        CatalogModule {
            uuid: Some(Uuid::from_u128(2)),
            bindings: None,
            versions: versions.iter().cloned().map(CatalogVersion::new).collect(),
        },
    );
    catalog
}

#[test]
fn discovering_many_dependencies_is_linear() {
    let catalog = one_module_catalog("@demo/module", &[ScriptVersion::new(1, u16::MAX, 0)]);
    let files = ScriptFiles::from_members(["main.js"]);
    let allowed = vec!["@demo/module".to_owned()];
    let cx = ScriptContext {
        allowed_modules: Some(&allowed),
        ..ScriptContext::default()
    };
    test_cost::assert_linear_at(
        "discover, n dependencies on one module",
        10_000,
        |n| {
            script_module(
                (0..n)
                    .map(|i| dependency_on("@demo/module", distinct(1, i), i))
                    .collect(),
            )
        },
        |module| {
            let found = ScriptDiscovery::new(&catalog).discover(module, &files, &cx);
            let plugin = found.plugin().expect("every request is served");
            assert_eq!(plugin.dependencies.len(), module.dependencies.len());
            assert!(found.diagnostics().is_empty());
        },
    );
}

#[test]
fn discovering_many_rejected_dependencies_is_linear() {
    // Every dependency is refused by the permission list and reported.
    let catalog = one_module_catalog("@demo/module", &[ScriptVersion::new(1, 0, 0)]);
    let files = ScriptFiles::from_members(["main.js"]);
    let allowed: Vec<String> = (0..64).map(|i| format!("@demo/allowed{i}")).collect();
    let cx = ScriptContext {
        allowed_modules: Some(&allowed),
        ..ScriptContext::default()
    };
    test_cost::assert_linear_at(
        "discover, n dependencies the permission list refuses",
        10_000,
        |n| {
            script_module(
                (0..n)
                    .map(|i| dependency_on("@demo/module", distinct(1, i), i))
                    .collect(),
            )
        },
        |module| {
            let found = ScriptDiscovery::new(&catalog).discover(module, &files, &cx);
            assert!(found.plugin().is_none());
            assert_eq!(found.diagnostics().len(), module.dependencies.len());
        },
    );
}

/// A catalog of `n` modules, each at 1.0.0 and requiring the next; with `cycle` the last
/// requires the first.
fn chain_catalog(n: usize, cycle: bool) -> ModuleCatalog {
    let mut catalog = ModuleCatalog::new();
    for i in 0..n {
        let mut version = CatalogVersion::new(ScriptVersion::new(1, 0, 0));
        if i + 1 < n {
            version.requires = vec![(format!("@chain/m{}", i + 1), ScriptVersion::new(1, 0, 0))];
        } else if cycle {
            version.requires = vec![("@chain/m0".to_owned(), ScriptVersion::new(1, 0, 0))];
        }
        catalog.insert(
            &format!("@chain/m{i}"),
            CatalogModule {
                uuid: Some(Uuid::from_u128(1000 + i as u128)),
                bindings: None,
                versions: vec![version],
            },
        );
    }
    catalog
}

#[test]
fn a_chain_and_a_cycle_of_requirements_are_walked_once() {
    let files = ScriptFiles::from_members(["main.js"]);
    let module = script_module(vec![dependency_on(
        "@chain/m0",
        ScriptVersion::new(1, 0, 0),
        0,
    )]);
    let walk = |catalog: &ModuleCatalog| {
        let found =
            ScriptDiscovery::new(catalog).discover(&module, &files, &ScriptContext::default());
        assert!(found.diagnostics().is_empty(), "{:?}", found.diagnostics());
        let plugin = found.plugin().expect("the chain resolves");
        assert_eq!(plugin.dependencies.len(), 1);
    };
    test_cost::assert_linear_at(
        "discover, a chain of n requiring modules",
        5_000,
        |n| chain_catalog(n, false),
        walk,
    );
    test_cost::assert_linear_at(
        "discover, a cycle of n requiring modules",
        5_000,
        |n| chain_catalog(n, true),
        walk,
    );
}

#[test]
fn reading_the_dependency_texts_is_linear() {
    test_cost::assert_linear_at(
        "dependency_texts, n dependencies",
        5_000,
        |n| {
            let dependencies: Vec<String> = (0..n)
                .map(|i| {
                    format!(
                        r#"{{ "module_name": "@demo/m{i}", "version": "2.{}", "optional": true }}"#,
                        i % 7
                    )
                })
                .collect();
            format!(r#"{{ "dependencies": [{}] }}"#, dependencies.join(","))
        },
        |text| {
            let texts = dependency_texts(text.as_bytes(), false);
            assert!(
                texts
                    .native
                    .iter()
                    .all(|native| native.relaxed && native.has_optional)
            );
            assert!(texts.packs.is_empty());
            texts.native.len()
        },
    );
}

// ---- where the resolver departs from vanilla's reports ----

#[test]
fn one_conflict_per_later_request_deviation() {
    // Vanilla reports every conflicting pair: three for requests in three majors. The
    // crate reports each later request once, with the earliest request it conflicts with.
    let (pairs, missing) = failed(&[own("1.19.0"), own("2.8.0"), own("3.0.0-alpha")]);
    assert_eq!(
        pairs,
        vec![(v("1.19.0"), v("2.8.0")), (v("1.19.0"), v("3.0.0-alpha")),]
    );
    assert_eq!(missing, vec![]);
    // And a request made twice by the same kind of asker is one row: vanilla prints the
    // conflict once per bindings version that asks, the crate once.
    let (pairs, _) = failed(&[own("1.19.0"), transitive("2.0.0"), transitive("2.0.0")]);
    assert_eq!(pairs, vec![(v("1.19.0"), v("2.0.0"))]);
}

#[test]
fn a_request_a_newer_version_serves_is_not_missing_deviation() {
    // Vanilla's "missing" is "the engine does not have this version", and its engine has
    // every release. A catalog may list the newest release of a major only (as the dedicated
    // server prints it), so the crate's is "no version serves the request": 1.5.0 is served by 1.19.0 and is not
    // reported beside the conflict.
    let rows = table(&[own("1.5.0"), own("9.0.0")], &server_versions());
    let missing: Vec<(String, bool)> = rows
        .iter()
        .filter(|row| !row.promotion)
        .map(|row| (row.version.to_string(), row.missing))
        .collect();
    assert_eq!(
        missing,
        vec![("1.5.0".to_owned(), false), ("9.0.0".to_owned(), true)]
    );
    // The same against a catalog that lists every release.
    let full = ["1.5.0", "1.19.0", "2.8.0"].map(v);
    let rows = table(&[own("1.5.0"), own("9.0.0")], &full);
    assert!(
        !rows
            .iter()
            .any(|row| row.version == v("1.5.0") && row.missing)
    );
    assert!(
        rows.iter()
            .any(|row| row.version == v("9.0.0") && row.missing)
    );
}

#[test]
fn script_files_collect_sorted_and_deduplicated() {
    let files: super::ScriptFiles = ["b.js", "a.js", "b.js"].into_iter().collect();
    assert_eq!(files.len(), 2);
    assert_eq!(files, super::ScriptFiles::from_members(["a.js", "b.js"]));
}

#[test]
fn module_catalog_collects_and_extends_by_insert() {
    use super::{CatalogModule, ModuleCatalog};
    let module = |uuid| CatalogModule {
        uuid: Some(Uuid::from_u128(uuid)),
        ..CatalogModule::default()
    };
    let mut catalog: ModuleCatalog = [("@a/one".to_owned(), module(1))].into_iter().collect();
    catalog.extend([("@a/two".to_owned(), module(2))]);
    let mut by_insert = ModuleCatalog::new();
    by_insert.insert("@a/one", module(1));
    by_insert.insert("@a/two", module(2));
    assert_eq!(catalog, by_insert);
}
