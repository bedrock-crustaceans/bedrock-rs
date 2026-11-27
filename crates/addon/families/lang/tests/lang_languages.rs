//! Text tests that need both families, `texts/*.lang` and `texts/languages.json`: the stack
//! rules (translations keyed by the language list, pack keywords, the lints `languages()`
//! reports), the locale overlay and the lang document as loaded content.

#[path = "lang_languages/common.rs"]
mod common;

#[path = "lang_languages/content.rs"]
mod content;
