//! Text tests that need both families, `texts/*.lang` and `texts/languages.json`: the stack
//! rules (translations keyed by the language list, pack keywords, the lints `languages()`
//! reports), the locale overlay and the lang document as loaded content.

#![cfg(all(feature = "lang", feature = "languages"))]

#[path = "../../families/lang/tests/lang_languages/common.rs"]
mod common;

#[path = "common/responder.rs"]
mod responder;

#[path = "common/requests.rs"]
mod requests;

#[path = "lang_languages/stack.rs"]
mod stack;

#[path = "lang_languages/stacks.rs"]
mod stacks;

#[path = "lang_languages/overlay.rs"]
mod overlay;
