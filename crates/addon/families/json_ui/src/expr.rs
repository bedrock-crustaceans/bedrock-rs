//! The expressions of `binding_name`, `source_property_name` and `requires`: the client's
//! `ExprToken` tokens and a small evaluator over them.

use bedrock_addon_core::json::Value;

/// One token of an expression.
// The numbers after each operator name are the game's token values.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    Open,
    Close,
    /// `and` (1), `or` (2), `not` (10); `>` (3), `<` (4), `=` (5), `+` (6), `-` (7), `*` (8),
    /// `/` (9).
    Operator(Operator),
    Number(f64),
    /// A `'…'` literal, or a bare word that is no operator.
    Text(String),
    /// `$name`
    Variable(String),
    /// `#name`
    Binding(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operator {
    And,
    Or,
    Not,
    Greater,
    Less,
    Equal,
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// What the tokenizer finds wrong with an expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Issue {
    /// A `'` without its closing quote.
    UnclosedQuote,
    /// A `#` that starts no binding name.
    BareHash,
    /// `<=`, `>=` or `%`, which are not operators.
    UnsupportedOperator(&'static str),
}

/// The tokens of `text` and what is wrong with it.
pub(crate) fn tokenize(text: &str) -> (Vec<Token>, Vec<Issue>) {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut issues = Vec::new();
    let name_end = |from: usize| {
        from + bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'|'))
            .count()
    };
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'(' => tokens.push(Token::Open),
            b')' => tokens.push(Token::Close),
            b'\'' => {
                let Some(length) = bytes[at + 1..].iter().position(|b| *b == b'\'') else {
                    issues.push(Issue::UnclosedQuote);
                    break;
                };
                tokens.push(Token::Text(text[at + 1..at + 1 + length].to_owned()));
                at += length + 1;
            }
            b'$' | b'#' => {
                let end = name_end(at + 1);
                let name = text[at..end].to_owned();
                if end == at + 1 && byte == b'#' {
                    issues.push(Issue::BareHash);
                } else if byte == b'$' {
                    tokens.push(Token::Variable(name));
                } else {
                    tokens.push(Token::Binding(name));
                }
                at = end - 1;
            }
            b'>' | b'<' if bytes.get(at + 1) == Some(&b'=') => {
                issues.push(Issue::UnsupportedOperator(if byte == b'>' {
                    ">="
                } else {
                    "<="
                }));
                at += 1;
            }
            b'%' => issues.push(Issue::UnsupportedOperator("%")),
            b'>' => tokens.push(Token::Operator(Operator::Greater)),
            b'<' => tokens.push(Token::Operator(Operator::Less)),
            b'=' => tokens.push(Token::Operator(Operator::Equal)),
            b'+' => tokens.push(Token::Operator(Operator::Add)),
            b'-' => tokens.push(Token::Operator(Operator::Subtract)),
            b'*' => tokens.push(Token::Operator(Operator::Multiply)),
            b'/' => tokens.push(Token::Operator(Operator::Divide)),
            b'0'..=b'9' | b'.' => {
                let end = at
                    + bytes[at..]
                        .iter()
                        .take_while(|b| b.is_ascii_digit() || **b == b'.')
                        .count();
                tokens.push(Token::Number(text[at..end].parse().unwrap_or(0.0)));
                at = end - 1;
            }
            _ if byte.is_ascii_alphabetic() || byte == b'_' => {
                let end = name_end(at);
                tokens.push(match &text[at..end] {
                    "and" => Token::Operator(Operator::And),
                    "or" => Token::Operator(Operator::Or),
                    "not" => Token::Operator(Operator::Not),
                    word => Token::Text(word.to_owned()),
                });
                at = end - 1;
            }
            _ => {}
        }
        at += 1;
    }
    (tokens, issues)
}

/// A value an expression works on.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Operand {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    /// An array or an object: only its emptiness counts.
    Other(bool),
}

impl Operand {
    pub(crate) fn from_value(value: &Value) -> Self {
        if let Some(flag) = value.as_bool() {
            Self::Bool(flag)
        } else if let Some(number) = value.as_number() {
            Self::Number(number.to_f64_lossy())
        } else if let Some(text) = value.as_string() {
            Self::Text(text.as_str().to_owned())
        } else if let Some(array) = value.as_array() {
            Self::Other(!array.is_empty())
        } else if let Some(object) = value.as_object() {
            Self::Other(!object.is_empty())
        } else {
            Self::Null
        }
    }

    pub(crate) fn into_value(self) -> Value {
        match self {
            Self::Null | Self::Other(false) => Value::NULL,
            Self::Bool(flag) | Self::Other(flag) => Value::from(flag),
            Self::Number(number) => Value::from(number),
            Self::Text(text) => Value::from(text),
        }
    }

    /// Numeric and not 0, a non-empty string, or a non-empty other value.
    pub(crate) fn truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(flag) | Self::Other(flag) => *flag,
            Self::Number(number) => *number != 0.0,
            Self::Text(text) => !text.is_empty(),
        }
    }

    fn as_number(&self) -> f64 {
        match self {
            Self::Number(number) => *number,
            Self::Bool(true) => 1.0,
            Self::Text(text) => text.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }
}

/// Evaluates `text` with `variable` giving the value of each `$name`; a `#binding` has no
/// value here. The loosest operator is `or`, then `and`, `not`, the comparisons, `+ -`, and
/// `* /`.
pub(crate) fn evaluate(text: &str, variable: &dyn Fn(&str) -> Option<Value>) -> Operand {
    let (tokens, _) = tokenize(text);
    let mut parser = Parser {
        tokens: &tokens,
        at: 0,
        variable,
        depth: 0,
    };
    parser.level(0)
}

struct Parser<'a> {
    tokens: &'a [Token],
    at: usize,
    variable: &'a dyn Fn(&str) -> Option<Value>,
    depth: usize,
}

/// The binary operators by level, loosest first.
const LEVELS: [&[Operator]; 5] = [
    &[Operator::Or],
    &[Operator::And],
    &[Operator::Greater, Operator::Less, Operator::Equal],
    &[Operator::Add, Operator::Subtract],
    &[Operator::Multiply, Operator::Divide],
];
/// `not` binds tighter than `and` and looser than a comparison.
const NOT_LEVEL: usize = 2;
/// Parentheses and `not` nest only this deep; deeper input evaluates to nothing.
const MAX_NESTING: usize = 64;

impl Parser<'_> {
    fn level(&mut self, level: usize) -> Operand {
        if level == NOT_LEVEL && self.tokens.get(self.at) == Some(&Token::Operator(Operator::Not)) {
            self.at += 1;
            self.depth += 1;
            let operand = if self.depth > MAX_NESTING {
                Operand::Null
            } else {
                self.level(level)
            };
            self.depth -= 1;
            return Operand::Bool(!operand.truthy());
        }
        let Some(operators) = LEVELS.get(level) else {
            return self.primary();
        };
        let mut left = self.level(level + 1);
        while let Some(Token::Operator(operator)) = self.tokens.get(self.at) {
            if !operators.contains(operator) {
                break;
            }
            let operator = *operator;
            self.at += 1;
            let right = self.level(level + 1);
            left = apply(operator, &left, &right);
        }
        left
    }

    fn primary(&mut self) -> Operand {
        let token = self.tokens.get(self.at);
        self.at += 1;
        match token {
            Some(Token::Open) => {
                self.depth += 1;
                let inner = if self.depth > MAX_NESTING {
                    Operand::Null
                } else {
                    self.level(0)
                };
                self.depth -= 1;
                if self.tokens.get(self.at) == Some(&Token::Close) {
                    self.at += 1;
                }
                inner
            }
            Some(Token::Number(number)) => Operand::Number(*number),
            Some(Token::Text(text)) => match text.as_str() {
                "true" => Operand::Bool(true),
                "false" => Operand::Bool(false),
                _ => Operand::Text(text.clone()),
            },
            Some(Token::Variable(name)) => {
                (self.variable)(name).map_or(Operand::Null, |value| Operand::from_value(&value))
            }
            Some(Token::Operator(Operator::Subtract)) => {
                // A run of signs is counted, not recursed into: its length is the input's.
                let mut negative = true;
                while self.tokens.get(self.at) == Some(&Token::Operator(Operator::Subtract)) {
                    self.at += 1;
                    negative = !negative;
                }
                let number = match self.tokens.get(self.at) {
                    Some(Token::Operator(_)) | None => 0.0,
                    Some(_) => self.primary().as_number(),
                };
                Operand::Number(if negative { -number } else { number })
            }
            _ => Operand::Null,
        }
    }
}

fn apply(operator: Operator, left: &Operand, right: &Operand) -> Operand {
    match operator {
        Operator::Or => Operand::Bool(left.truthy() || right.truthy()),
        Operator::And => Operand::Bool(left.truthy() && right.truthy()),
        Operator::Greater => Operand::Bool(left.as_number() > right.as_number()),
        Operator::Less => Operand::Bool(left.as_number() < right.as_number()),
        Operator::Equal => Operand::Bool(match (left, right) {
            (Operand::Text(a), Operand::Text(b)) => a == b,
            (Operand::Text(_), _) | (_, Operand::Text(_)) => false,
            _ => (left.as_number() - right.as_number()).abs() < f64::EPSILON,
        }),
        Operator::Add => match (left, right) {
            (Operand::Text(a), Operand::Text(b)) => Operand::Text(format!("{a}{b}")),
            _ => Operand::Number(left.as_number() + right.as_number()),
        },
        Operator::Subtract => Operand::Number(left.as_number() - right.as_number()),
        Operator::Multiply => Operand::Number(left.as_number() * right.as_number()),
        Operator::Divide => Operand::Number(left.as_number() / right.as_number()),
        Operator::Not => Operand::Bool(!right.truthy()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(text: &str, set: &[(&str, bool)]) -> Operand {
        evaluate(text, &|name| {
            set.iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| Value::from(*v))
        })
    }

    #[test]
    fn words_and_single_characters_are_the_operators() {
        let (tokens, issues) = tokenize("(#a > 2 and not #b)");
        assert!(issues.is_empty());
        assert_eq!(
            tokens,
            vec![
                Token::Open,
                Token::Binding("#a".to_owned()),
                Token::Operator(Operator::Greater),
                Token::Number(2.0),
                Token::Operator(Operator::And),
                Token::Operator(Operator::Not),
                Token::Binding("#b".to_owned()),
                Token::Close,
            ]
        );
    }

    #[test]
    fn two_character_comparisons_and_percent_are_no_operators() {
        assert_eq!(
            tokenize("(#a >= 2)").1,
            vec![Issue::UnsupportedOperator(">=")]
        );
        assert_eq!(
            tokenize("(#a <= 2)").1,
            vec![Issue::UnsupportedOperator("<=")]
        );
        assert_eq!(
            tokenize("(#a % 2)").1,
            vec![Issue::UnsupportedOperator("%")]
        );
        // inside quotes a `%` is text: format strings are ordinary string tokens
        assert_eq!(tokenize("('%.1s' + #a)").1, vec![]);
    }

    #[test]
    fn an_open_quote_and_a_bare_hash_are_reported() {
        assert_eq!(tokenize("(#a = 'x)").1, vec![Issue::UnclosedQuote]);
        assert_eq!(tokenize("(# = 1)").1, vec![Issue::BareHash]);
        assert_eq!(tokenize("('#' = #a)").1, vec![]);
    }

    #[test]
    fn not_binds_tighter_than_and_and_or() {
        let set = [("$a", true), ("$b", false)];
        assert_eq!(eval("(not $b and $a)", &set), Operand::Bool(true));
        assert_eq!(eval("(not $a or $a)", &set), Operand::Bool(true));
        assert_eq!(eval("(not ($a or $b))", &set), Operand::Bool(false));
        assert_eq!(eval("($missing)", &set), Operand::Null);
    }

    #[test]
    fn arithmetic_and_comparisons_evaluate() {
        assert_eq!(eval("(1 + 2 * 3)", &[]), Operand::Number(7.0));
        assert_eq!(eval("(1 + 2 > 2)", &[]), Operand::Bool(true));
        assert_eq!(eval("('a' = 'a')", &[]), Operand::Bool(true));
        assert_eq!(eval("('a' = 1)", &[]), Operand::Bool(false));
        assert_eq!(eval("(2 < 1)", &[]), Operand::Bool(false));
        assert_eq!(eval("(6 / 3 - 1)", &[]), Operand::Number(1.0));
    }

    #[test]
    fn a_long_run_of_signs_ends() {
        let text = format!("({}1)", "- ".repeat(200_000));
        assert_eq!(eval(&text, &[]), Operand::Number(1.0));
        let text = format!("({}1)", "- ".repeat(200_001));
        assert_eq!(eval(&text, &[]), Operand::Number(-1.0));
    }

    /// Ill-formed input the game does not define, such as a sign followed by another
    /// operator: pinned as it behaves here, not as a rule of the game's.
    #[test]
    fn a_sign_before_another_operator_is_unspecified() {
        // printed, because `-0` and `0` are equal as numbers
        let results: Vec<String> = ["(- + 1)", "(- - + 1)", "(- * 2)", "(-)", "(1 - + 2)"]
            .iter()
            .map(|text| format!("{:?}", eval(text, &[])))
            .collect();
        assert_eq!(
            results,
            [
                "Number(1.0)",
                "Number(1.0)",
                "Number(-0.0)",
                "Number(-0.0)",
                "Number(1.0)"
            ]
        );
    }

    #[test]
    fn deep_nesting_ends() {
        let text = format!("{}1{}", "(".repeat(5000), ")".repeat(5000));
        assert_eq!(eval(&text, &[]), Operand::Null);
        let text = "not ".repeat(5000);
        let _ = eval(&text, &[]);
    }
}
