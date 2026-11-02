//! The layout axis grammar of `size`, `offset`, `min_size` and `max_size`
//! as the game parses them.

use bedrock_addon_core::json::Value;

/// One `+`/`-` joined term of an axis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Term {
    /// The term is subtracted.
    pub negative: bool,
    /// What the number is measured in.
    pub unit: Unit,
}

/// The unit of a term.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unit {
    /// A JSON number: pixels, and as an animation endpoint a match for any unit.
    Number,
    /// `Npx`.
    Pixels,
    /// `N%` with its modifier letters (`c` children, `m` max, `s` sibling, `x`/`y` the other
    /// axis).
    Percent(String),
    /// `default` or `fill`.
    Keyword,
}

/// Why an axis does not parse; each is one of the game's messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AxisError {
    /// `Dangling number (no % or px in Size)`
    DanglingNumber,
    /// `% modifier given without '%' symbol!`
    ModifierWithoutPercent,
    /// `An unknown sibling axis type was found: <letters>`
    UnknownSiblingAxis(String),
    /// `Unrecognized token!`
    UnrecognizedToken,
    /// `Trying to give JSON value without a number value!`
    NotANumber,
}

impl AxisError {
    /// The game's text. Its format strings spell `%` as `%%`; this is what they print.
    pub(crate) fn text(&self) -> (&'static str, Vec<String>) {
        match self {
            Self::DanglingNumber => ("Dangling number (no % or px in Size)", Vec::new()),
            Self::ModifierWithoutPercent => ("% modifier given without '%' symbol!", Vec::new()),
            Self::UnknownSiblingAxis(letters) => (
                "An unknown sibling axis type was found: {}",
                vec![letters.clone()],
            ),
            Self::UnrecognizedToken => ("Unrecognized token!", Vec::new()),
            Self::NotANumber => (
                "Trying to give JSON value without a number value!",
                Vec::new(),
            ),
        }
    }
}

/// The letters the game scans after a `%`.
const MODIFIERS: &[u8] = b"cmswxy";

/// The terms of one axis: a JSON number (pixels) or a string of `+`/`-` joined terms.
// A number is one pixel term; a string is split into its signed terms, each read on its own.
pub(crate) fn parse_axis(value: &Value) -> Result<Vec<Term>, AxisError> {
    if value.is_number() {
        return Ok(vec![Term {
            negative: false,
            unit: Unit::Number,
        }]);
    }
    let Some(text) = value.as_string() else {
        return Err(AxisError::NotANumber);
    };
    let text = text.as_str().trim();
    if matches!(text, "default" | "fill") {
        return Ok(vec![Term {
            negative: false,
            unit: Unit::Keyword,
        }]);
    }
    let bytes = text.as_bytes();
    let mut terms = Vec::new();
    let mut at = 0;
    let skip_spaces = |at: &mut usize| {
        while bytes.get(*at) == Some(&b' ') {
            *at += 1;
        }
    };
    loop {
        skip_spaces(&mut at);
        // The sign before the first term is the number's; between terms it is the operation.
        let mut negative = false;
        if let Some(sign @ (b'+' | b'-')) = bytes.get(at) {
            negative = *sign == b'-' && !terms.is_empty();
            at += 1;
            skip_spaces(&mut at);
        }
        let digits = bytes[at..]
            .iter()
            .take_while(|b| b.is_ascii_digit() || **b == b'.')
            .count();
        if digits == 0 {
            return Err(AxisError::UnrecognizedToken);
        }
        at += digits;
        let letters = |from: usize| {
            bytes[from..]
                .iter()
                .take_while(|b| b.is_ascii_alphabetic())
                .count()
        };
        let unit = match bytes.get(at) {
            Some(b'%') => {
                let count = letters(at + 1);
                let modifier = &text[at + 1..at + 1 + count];
                if !modifier.bytes().all(|b| MODIFIERS.contains(&b)) {
                    return Err(AxisError::UnknownSiblingAxis(modifier.to_owned()));
                }
                at += 1 + count;
                Unit::Percent(modifier.to_owned())
            }
            Some(b'p') if bytes.get(at + 1) == Some(&b'x') => {
                at += 2;
                Unit::Pixels
            }
            None | Some(b' ' | b'+' | b'-') => return Err(AxisError::DanglingNumber),
            Some(letter) if MODIFIERS.contains(letter) => {
                return Err(AxisError::ModifierWithoutPercent);
            }
            Some(_) => return Err(AxisError::UnrecognizedToken),
        };
        terms.push(Term { negative, unit });
        skip_spaces(&mut at);
        match bytes.get(at) {
            None => return Ok(terms),
            Some(b'+' | b'-') => {}
            Some(_) => return Err(AxisError::UnrecognizedToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis(text: &str) -> Result<Vec<Term>, AxisError> {
        parse_axis(&Value::from(text))
    }

    #[test]
    fn the_vanilla_forms_parse() {
        for form in [
            "100%",
            "100%c",
            "100%cm",
            "100%sm",
            "100%x",
            "100%y",
            "100% - 4px",
            "100%c + 2px",
            "default",
            "fill",
            "-2px",
            "12.5%",
        ] {
            assert!(axis(form).is_ok(), "{form}");
        }
        assert_eq!(
            parse_axis(&Value::from(4_i64)).unwrap()[0].unit,
            Unit::Number
        );
        // a leading minus is the number's sign, not an operation
        assert!(!axis("-100%").unwrap()[0].negative);
        assert!(!axis("+2px").unwrap()[0].negative);
        let terms = axis("100%c - 4px").unwrap();
        assert_eq!(terms[0].unit, Unit::Percent("c".to_owned()));
        assert!(terms[1].negative && terms[1].unit == Unit::Pixels);
    }

    #[test]
    fn each_malformed_axis_has_its_message() {
        assert_eq!(axis("100"), Err(AxisError::DanglingNumber));
        assert_eq!(axis("100 + 2px"), Err(AxisError::DanglingNumber));
        assert_eq!(axis("50c"), Err(AxisError::ModifierWithoutPercent));
        assert_eq!(
            axis("100%q"),
            Err(AxisError::UnknownSiblingAxis("q".to_owned()))
        );
        assert_eq!(axis("wide"), Err(AxisError::UnrecognizedToken));
        assert_eq!(axis("10px 5px"), Err(AxisError::UnrecognizedToken));
        assert_eq!(axis("10k"), Err(AxisError::UnrecognizedToken));
        // `px` is the whole unit
        assert_eq!(axis("10pt"), Err(AxisError::UnrecognizedToken));
        assert_eq!(axis("10p"), Err(AxisError::UnrecognizedToken));
        assert_eq!(parse_axis(&Value::from(true)), Err(AxisError::NotANumber));
    }
}
