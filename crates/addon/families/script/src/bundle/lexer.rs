//! The import lexer: one pass over a script file with the ECMAScript lexical grammar (module
//! goal), looking only at token sequences, so text inside comments, strings, templates and
//! regular expressions never matches.
// No recursion: the open
// brackets and template substitutions are a `Vec`; the cost is linear in the file.

use super::{ImportKind, ScriptPosition, ScriptSyntaxKind};

/// An import site as written, before its specifier is resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RawImport {
    pub(super) kind: ImportKind,
    /// The specifier, escapes decoded; empty for `DynamicNonLiteral`.
    pub(super) raw: String,
    /// The opening quote (`import` for `DynamicNonLiteral`).
    pub(super) at: ScriptPosition,
    /// `with { … }` / `assert { … }` follows.
    pub(super) attributes: bool,
}

/// What the lexer found in one file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Lexed {
    /// The import sites, in file order.
    pub(super) imports: Vec<RawImport>,
    /// `eval(` and `Function(` call tokens.
    pub(super) evals: Vec<ScriptPosition>,
    /// `import.meta` followed by a member read.
    pub(super) metas: Vec<ScriptPosition>,
    /// The error that stopped the scan.
    pub(super) error: Option<(ScriptSyntaxKind, ScriptPosition)>,
}

/// What the token before the current one allows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prev {
    /// The end of an expression: `/` divides, `{` opens a block.
    ExprEnd,
    /// The start of a statement (`;`, a block's `{` or `}`, `=>`, `else`): `/` starts a
    /// regular expression, `{` opens a block.
    Stmt,
    /// An operator or an opening bracket: `/` starts a regular expression, `{` an object.
    Op,
    /// `:`: `/` starts a regular expression; `{` is an object inside an object or a bracket,
    /// else a block (a label, a `case`).
    Colon,
}

/// An open bracket.
#[derive(Clone, Copy)]
enum Open {
    /// `(`; `control` after `if`, `while`, `for`, `with`: its `)` ends no expression.
    /// `for_head` after `for` and `for await`: an `of` in it is the keyword. `import` names
    /// the site of the `import(` it opens.
    Paren {
        control: bool,
        for_head: bool,
        import: Option<usize>,
    },
    /// `[`.
    Bracket,
    /// `{`; `object` for an object literal: its `}` ends an expression.
    Brace { object: bool },
    /// `${` of a template literal that starts at the position.
    Template(ScriptPosition),
}

/// A token, as the import rules see it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tok<'a> {
    /// An identifier or keyword that is not a property name.
    Word(&'a str),
    /// A string literal, or a template literal without substitutions: its content between
    /// the quotes, as written.
    Text(&'a str),
    /// A property name: a word after `.` or `?.`.
    Prop(&'a str),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    Dot,
    Comma,
    Star,
    /// Anything else.
    Other,
}

/// Where the import rules are between two tokens.
#[derive(Clone, PartialEq, Eq)]
enum State {
    Idle,
    /// After `import`.
    Import(ScriptPosition),
    /// In the clause of an `import` or an `export`, `depth` braces deep. `first` is the
    /// clause's first word while nothing else has followed it (`defer`, `source`).
    Clause {
        kind: ImportKind,
        depth: u32,
        from: bool,
        first: Option<bool>,
    },
    /// After `export`.
    Export,
    /// After the specifier of the static import `site`.
    Spec(usize),
    /// After `with` / `assert` behind the specifier of `site`.
    With(usize),
    /// After `import.`.
    MetaDot(ScriptPosition),
    /// After `import.meta`.
    Meta(ScriptPosition),
    /// After `import(`; `paren` is the index of that `(` among the open brackets.
    Call {
        import: ScriptPosition,
        paren: usize,
    },
    /// After `import("literal"`.
    Arg {
        import: ScriptPosition,
        paren: usize,
        at: ScriptPosition,
        raw: String,
    },
    /// After the `)` of the `import(…)` of `site`: a `{` makes it a method named `import`.
    Closed(usize),
    /// After `eval` or `Function`.
    Eval(ScriptPosition),
}

/// Line and column of offsets asked for in rising order: every byte is counted once.
struct Locator<'a> {
    source: &'a str,
    offset: usize,
    line: u32,
    column: u32,
}

impl Locator<'_> {
    fn at(&mut self, offset: usize) -> ScriptPosition {
        let offset = offset.clamp(self.offset, self.source.len());
        for byte in &self.source.as_bytes()[self.offset..offset] {
            if *byte == b'\n' {
                self.line = self.line.saturating_add(1);
                self.column = 1;
            } else if *byte & 0xC0 != 0x80 {
                // The first byte of a character.
                self.column = self.column.saturating_add(1);
            }
        }
        self.offset = offset;
        ScriptPosition {
            offset: offset as u64,
            line: self.line,
            column: self.column,
        }
    }
}

/// Words after which `/` divides: they end an expression.
fn ends_expression(word: &str) -> bool {
    matches!(word, "this" | "super" | "null" | "true" | "false")
}

/// Reserved words that are no expression: after one, `/` starts a regular expression.
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "switch"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

/// Keywords a block follows, never an object literal.
fn opens_block(word: &str) -> bool {
    matches!(word, "else" | "do" | "try" | "finally" | "class" | "export")
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte == b'\\' || byte >= 0x80
}

/// The length of the white space at the start of `rest`, the Unicode spaces and line
/// terminators of the grammar included.
fn space_len(rest: &[u8]) -> usize {
    match rest {
        [b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C, ..] => 1,
        // U+00A0.
        [0xC2, 0xA0, ..] => 2,
        // U+2028, U+2029, U+FEFF.
        [0xE2, 0x80, 0xA8 | 0xA9, ..] | [0xEF, 0xBB, 0xBF, ..] => 3,
        _ => 0,
    }
}

/// `rest` starts with a line terminator.
fn is_line_end(rest: &[u8]) -> bool {
    matches!(rest, [b'\n' | b'\r', ..] | [0xE2, 0x80, 0xA8 | 0xA9, ..])
}

/// The cooked value of a string or template literal's content.
// Rule 9: the engine sees the cooked value.
pub(super) fn cook(raw: &str) -> String {
    if !raw.contains('\\') {
        return raw.to_owned();
    }
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    let hex = |chars: &mut std::iter::Peekable<std::str::Chars<'_>>, digits: usize| {
        let mut value = 0u32;
        for _ in 0..digits {
            let digit = chars.peek().and_then(|c| c.to_digit(16))?;
            chars.next();
            value = value * 16 + digit;
        }
        Some(value)
    };
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('v') => out.push('\u{b}'),
            Some('0') => out.push('\0'),
            // A line continuation.
            Some('\r') => {
                chars.next_if_eq(&'\n');
            }
            None | Some('\n' | '\u{2028}' | '\u{2029}') => {}
            Some('x') => out.push(
                hex(&mut chars, 2)
                    .and_then(char::from_u32)
                    .unwrap_or(char::REPLACEMENT_CHARACTER),
            ),
            Some('u') => {
                let unit = if chars.next_if_eq(&'{').is_some() {
                    let mut value = 0u32;
                    while let Some(digit) = chars.peek().and_then(|c| c.to_digit(16)) {
                        chars.next();
                        value = value.saturating_mul(16).saturating_add(digit);
                    }
                    chars.next_if_eq(&'}');
                    Some(value)
                } else {
                    hex(&mut chars, 4)
                };
                let mut value = unit.unwrap_or(0xFFFD);
                // A surrogate pair written as two escapes.
                if (0xD800..0xDC00).contains(&value) {
                    let mut ahead = chars.clone();
                    let low = (ahead.next() == Some('\\') && ahead.next() == Some('u'))
                        .then(|| hex(&mut ahead, 4))
                        .flatten()
                        .filter(|low| (0xDC00..0xE000).contains(low));
                    if let Some(low) = low {
                        value = 0x10000 + ((value - 0xD800) << 10) + (low - 0xDC00);
                        chars = ahead;
                    }
                }
                out.push(char::from_u32(value).unwrap_or(char::REPLACEMENT_CHARACTER));
            }
            Some(other) => out.push(other),
        }
    }
    out
}

/// The lexer's state.
struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    prev: Prev,
    /// The word before the current token, when it is no property name.
    prev_word: &'a str,
    /// The token before the current one is `.` or `?.`.
    after_dot: bool,
    open: Vec<Open>,
    state: State,
    locator: Locator<'a>,
    out: Lexed,
    /// Sites that turned out to be methods named `import`.
    dropped: Vec<usize>,
    /// A line ended between the token before and the current one.
    line_break: bool,
    /// The token before is the name a `var` or a `let` declares.
    declared: bool,
    /// A scan for the end of a regular expression ran from before this offset to the end of
    /// the line it is in and found none.
    no_regex_before: usize,
}

/// Scans `source` (a script file, with its byte order mark if it has one) for import sites, `eval` /
/// `Function` calls and `import.meta` member reads.
pub(super) fn lex(source: &str) -> Lexed {
    let bom = if source.starts_with('\u{feff}') { 3 } else { 0 };
    let mut lexer = Lexer {
        source,
        bytes: source.as_bytes(),
        pos: bom,
        prev: Prev::Stmt,
        prev_word: "",
        after_dot: false,
        open: Vec::new(),
        state: State::Idle,
        locator: Locator {
            source,
            // A byte order mark is no character of the first line.
            offset: bom,
            line: 1,
            column: 1,
        },
        out: Lexed::default(),
        dropped: Vec::new(),
        line_break: false,
        declared: false,
        no_regex_before: 0,
    };
    lexer.run();
    let Lexer {
        mut out, dropped, ..
    } = lexer;
    if !dropped.is_empty() {
        let mut dead = vec![false; out.imports.len()];
        for site in dropped {
            dead[site] = true;
        }
        let mut dead = dead.into_iter();
        out.imports.retain(|_| !dead.next().unwrap_or(false));
    }
    out
}

impl<'a> Lexer<'a> {
    fn fail(&mut self, kind: ScriptSyntaxKind, offset: usize) {
        let at = self.locator.at(offset);
        self.out.error = Some((kind, at));
        self.pos = self.bytes.len();
    }

    fn run(&mut self) {
        // Rule 1: a leading `#!` line.
        if self.bytes[self.pos..].starts_with(b"#!") {
            self.line_comment();
        }
        while self.pos < self.bytes.len() && self.out.error.is_none() {
            self.step();
        }
        if self.out.error.is_none() {
            // A template whose substitution the file ends in.
            let unclosed = self.open.iter().find_map(|open| match open {
                Open::Template(start) => Some(*start),
                _ => None,
            });
            if let Some(start) = unclosed {
                self.out.error = Some((ScriptSyntaxKind::UnterminatedTemplate, start));
            }
        }
    }

    fn line_comment(&mut self) {
        while self.pos < self.bytes.len() {
            match &self.bytes[self.pos..] {
                [b'\n' | b'\r', ..] | [0xE2, 0x80, 0xA8 | 0xA9, ..] => break,
                _ => self.pos += 1,
            }
        }
    }

    /// One token, comment or run of white space.
    #[allow(
        clippy::too_many_lines,
        reason = "one arm per kind of token, each a few lines: the lexical grammar in one place"
    )]
    fn step(&mut self) {
        let rest = &self.bytes[self.pos..];
        let space = space_len(rest);
        if space > 0 {
            self.line_break |= is_line_end(rest);
            self.pos += space;
            return;
        }
        let start = self.pos;
        let byte = rest[0];
        match byte {
            b'/' if rest.get(1) == Some(&b'/') => self.line_comment(),
            b'/' if rest.get(1) == Some(&b'*') => match find(&rest[2..], *b"*/") {
                Some(end) => {
                    // A comment that holds a line end is one.
                    let text = &rest[2..2 + end];
                    self.line_break |= (0..text.len()).any(|at| is_line_end(&text[at..]));
                    self.pos += 2 + end + 2;
                }
                None => self.fail(ScriptSyntaxKind::UnterminatedComment, start),
            },
            b'/' => {
                // After a declared name and after an import's specifier only `=`, `,` or the
                // end of the statement can follow: on the next line a `/` starts a statement.
                let statement =
                    self.line_break && (self.declared || matches!(self.state, State::Spec(_)));
                let regex = if self.prev == Prev::ExprEnd && !statement {
                    None
                } else {
                    self.regex_len(start)
                };
                if let Some(len) = regex {
                    self.pos += len;
                    self.token(Tok::Other, start, Prev::ExprEnd);
                } else {
                    self.pos += 1;
                    self.token(Tok::Other, start, Prev::Op);
                }
            }
            b'\'' | b'"' => match string_len(rest) {
                Some(len) => {
                    self.pos += len;
                    let text = &self.source[start + 1..start + len - 1];
                    self.token(Tok::Text(text), start, Prev::ExprEnd);
                }
                None => self.fail(ScriptSyntaxKind::UnterminatedString, start),
            },
            b'`' => {
                let at = self.locator.at(start);
                self.template(at, start + 1, true);
            }
            b'0'..=b'9' => {
                self.pos += number_len(rest);
                self.token(Tok::Other, start, Prev::ExprEnd);
            }
            b'.' if rest.get(1).is_some_and(u8::is_ascii_digit) => {
                self.pos += number_len(rest);
                self.token(Tok::Other, start, Prev::ExprEnd);
            }
            b'.' if rest.starts_with(b"...") => {
                self.pos += 3;
                self.token(Tok::Other, start, Prev::Op);
            }
            b'.' => {
                self.pos += 1;
                self.token(Tok::Dot, start, Prev::Op);
            }
            b'?' if rest.get(1) == Some(&b'.') && !rest.get(2).is_some_and(u8::is_ascii_digit) => {
                self.pos += 2;
                self.token(Tok::Dot, start, Prev::Op);
            }
            b'#' => {
                // A private name.
                self.pos += 1;
                while self.pos < self.bytes.len() && is_word_byte(self.bytes[self.pos]) {
                    self.pos += 1;
                }
                self.token(Tok::Other, start, Prev::ExprEnd);
            }
            _ if is_word_byte(byte) => {
                while self.pos < self.bytes.len() && is_word_byte(self.bytes[self.pos]) {
                    self.pos += 1;
                }
                let word = &self.source[start..self.pos];
                if self.after_dot {
                    // A property name, whatever its spelling.
                    self.token(Tok::Prop(word), start, Prev::ExprEnd);
                } else {
                    // `for (a of /re/)`: the `of` of a `for` is followed by an expression.
                    let of = word == "of"
                        && matches!(self.open.last(), Some(Open::Paren { for_head: true, .. }));
                    let prev = if of {
                        Prev::Op
                    } else if ends_expression(word) || !is_keyword(word) {
                        Prev::ExprEnd
                    } else if opens_block(word) {
                        Prev::Stmt
                    } else {
                        Prev::Op
                    };
                    // `for await (`: the parenthesis is the `for`'s.
                    let head = if word == "await" && self.prev_word == "for" {
                        "for"
                    } else {
                        word
                    };
                    let declared = matches!(self.prev_word, "var" | "let") && !is_keyword(word);
                    self.token(Tok::Word(word), start, prev);
                    self.prev_word = head;
                    self.declared = declared;
                }
            }
            b'(' => {
                self.pos += 1;
                let control = matches!(self.prev_word, "if" | "while" | "for" | "with");
                self.open.push(Open::Paren {
                    control,
                    for_head: self.prev_word == "for",
                    import: None,
                });
                self.token(Tok::LParen, start, Prev::Op);
            }
            b')' => {
                self.pos += 1;
                let (control, import) = match self.open.last() {
                    Some(Open::Paren {
                        control, import, ..
                    }) => {
                        let closed = (*control, *import);
                        self.open.pop();
                        closed
                    }
                    _ => (false, None),
                };
                let prev = if control { Prev::Stmt } else { Prev::ExprEnd };
                self.token(Tok::RParen, start, prev);
                if let (Some(site), State::Idle) = (import, &self.state) {
                    self.state = State::Closed(site);
                }
            }
            b'[' => {
                self.pos += 1;
                self.open.push(Open::Bracket);
                self.token(Tok::LBracket, start, Prev::Op);
            }
            b']' => {
                self.pos += 1;
                if matches!(self.open.last(), Some(Open::Bracket)) {
                    self.open.pop();
                }
                self.token(Tok::Other, start, Prev::ExprEnd);
            }
            b'{' => {
                self.pos += 1;
                let object = match self.prev {
                    Prev::ExprEnd | Prev::Stmt => false,
                    Prev::Op => true,
                    Prev::Colon => !matches!(
                        self.open.last(),
                        None | Some(Open::Brace { object: false } | Open::Template(_))
                    ),
                };
                self.open.push(Open::Brace { object });
                self.token(Tok::LBrace, start, Prev::Stmt);
            }
            b'}' => match self.open.last() {
                Some(Open::Template(template)) => {
                    let template = *template;
                    self.open.pop();
                    self.template(template, start + 1, false);
                }
                top => {
                    let object = matches!(top, Some(Open::Brace { object: true }));
                    if matches!(top, Some(Open::Brace { .. })) {
                        self.open.pop();
                    }
                    self.pos += 1;
                    let prev = if object { Prev::ExprEnd } else { Prev::Stmt };
                    self.token(Tok::RBrace, start, prev);
                }
            },
            b';' => {
                self.pos += 1;
                self.token(Tok::Other, start, Prev::Stmt);
            }
            b':' => {
                self.pos += 1;
                self.token(Tok::Other, start, Prev::Colon);
            }
            b',' => {
                self.pos += 1;
                self.token(Tok::Comma, start, Prev::Op);
            }
            b'*' if rest.get(1) == Some(&b'*') => {
                self.pos += 2;
                self.token(Tok::Other, start, Prev::Op);
            }
            b'*' => {
                self.pos += 1;
                self.token(Tok::Star, start, Prev::Op);
            }
            b'=' if rest.get(1) == Some(&b'>') => {
                self.pos += 2;
                self.token(Tok::Other, start, Prev::Stmt);
            }
            b'+' | b'-' if rest.get(1) == Some(&byte) => {
                // `++` / `--`: postfix after an expression, else prefix.
                self.pos += 2;
                let prev = if self.prev == Prev::ExprEnd {
                    Prev::ExprEnd
                } else {
                    Prev::Op
                };
                self.token(Tok::Other, start, prev);
            }
            _ => {
                // Any other operator or punctuator, a byte at a time: an expression follows.
                self.pos += 1;
                self.token(Tok::Other, start, Prev::Op);
            }
        }
    }

    /// The length of the regular-expression literal at `start`, or `None` when its line ends
    /// first. A line is scanned to its end once: after a scan that found no end, a later `/`
    /// of the line is followed only to its closing `/` or to the next square bracket.
    // From that bracket on the scan would be in the state the first scan was in (inside a
    // class or not, escapes paired alike), and the first scan found no end.
    fn regex_len(&mut self, start: usize) -> Option<usize> {
        let known = start < self.no_regex_before;
        match regex_len(&self.bytes[start..], known) {
            Ok(len) => Some(len),
            Err(stop) => {
                if !known {
                    self.no_regex_before = start + stop;
                }
                None
            }
        }
    }

    /// Scans a template literal from `from` (after its backquote, or after the `}` that ends
    /// a substitution) to its closing backquote or its next `${`. `start` is the backquote;
    /// `whole` when no substitution came before `from`.
    fn template(&mut self, start_at: ScriptPosition, from: usize, whole: bool) {
        let start = usize::try_from(start_at.offset).unwrap_or(usize::MAX);
        let mut at = from;
        loop {
            match self.bytes.get(at) {
                None => {
                    self.out.error = Some((ScriptSyntaxKind::UnterminatedTemplate, start_at));
                    self.pos = self.bytes.len();
                    return;
                }
                Some(b'\\') => at += 2,
                Some(b'`') => {
                    self.pos = at + 1;
                    let tok = if whole {
                        Tok::Text(&self.source[start + 1..at])
                    } else {
                        Tok::Other
                    };
                    if whole {
                        self.token(tok, start, Prev::ExprEnd);
                    } else {
                        // The tail of a template the rules already saw the head of.
                        self.prev = Prev::ExprEnd;
                        self.prev_word = "";
                        self.after_dot = false;
                    }
                    return;
                }
                Some(b'$') if self.bytes.get(at + 1) == Some(&b'{') => {
                    self.pos = at + 2;
                    if whole {
                        self.token(Tok::Other, start, Prev::Op);
                    }
                    self.open.push(Open::Template(start_at));
                    self.prev = Prev::Op;
                    self.prev_word = "";
                    self.after_dot = false;
                    return;
                }
                Some(_) => at += 1,
            }
        }
    }

    /// Records a token: runs the import rules on it, then makes it the previous token.
    fn token(&mut self, tok: Tok<'a>, start: usize, prev: Prev) {
        self.rules(tok, start);
        self.prev = prev;
        self.prev_word = "";
        self.after_dot = tok == Tok::Dot;
        self.line_break = false;
        self.declared = false;
    }

    fn site(&mut self, kind: ImportKind, raw: String, at: ScriptPosition) -> usize {
        self.out.imports.push(RawImport {
            kind,
            raw,
            at,
            attributes: false,
        });
        self.out.imports.len() - 1
    }

    /// Ties the `(` at `paren` to the `import(` of `site`; `closed` when the current token
    /// is its `)`.
    fn mark_call(&mut self, site: usize, paren: usize, closed: bool) {
        if closed {
            self.state = State::Closed(site);
        } else if let Some(Open::Paren { import, .. }) = self.open.get_mut(paren) {
            *import = Some(site);
        }
    }

    /// The import rules (4 to 11), one token at a time.
    #[allow(
        clippy::too_many_lines,
        reason = "one arm per transition of the rules' state machine"
    )]
    fn rules(&mut self, tok: Tok<'a>, start: usize) {
        let state = std::mem::replace(&mut self.state, State::Idle);
        match (state, tok) {
            // Rule 4.
            (State::Import(_), Tok::Text(text)) => {
                let at = self.locator.at(start);
                let site = self.site(ImportKind::SideEffect, cook(text), at);
                self.state = State::Spec(site);
            }
            // Rule 6.
            (State::Import(import), Tok::LParen) => {
                self.state = State::Call {
                    import,
                    paren: self.open.len().saturating_sub(1),
                };
            }
            // Rule 7.
            (State::Import(at), Tok::Dot) => self.state = State::MetaDot(at),
            (State::Import(_), Tok::Word(word)) => {
                self.state = State::Clause {
                    kind: ImportKind::Static,
                    depth: 0,
                    from: false,
                    first: Some(matches!(word, "defer" | "source")),
                };
            }
            (State::Import(_), Tok::Star) => {
                self.state = State::Clause {
                    kind: ImportKind::Static,
                    depth: 0,
                    from: false,
                    first: None,
                };
            }
            (State::Import(_), Tok::LBrace) => {
                self.state = State::Clause {
                    kind: ImportKind::Static,
                    depth: 1,
                    from: false,
                    first: None,
                };
            }
            // Rule 5.
            (State::Export, Tok::Star) => {
                self.state = State::Clause {
                    kind: ImportKind::ReExport,
                    depth: 0,
                    from: false,
                    first: None,
                };
            }
            (State::Export, Tok::LBrace) => {
                self.state = State::Clause {
                    kind: ImportKind::ReExport,
                    depth: 1,
                    from: false,
                    first: None,
                };
            }
            (
                State::Clause {
                    kind,
                    depth: 0,
                    from: true,
                    ..
                },
                Tok::Text(text),
            ) => {
                let at = self.locator.at(start);
                let site = self.site(kind, cook(text), at);
                self.state = State::Spec(site);
            }
            (State::Clause { depth: 0, .. }, Tok::Word("import" | "export")) => {
                self.fresh(tok, start);
            }
            (
                State::Clause {
                    kind, depth, first, ..
                },
                Tok::Word(_) | Tok::Star | Tok::Comma | Tok::LBrace | Tok::RBrace | Tok::Text(_),
            ) => {
                // Rule 10: `import defer x`, `import source x`, `import defer * as ns` are
                // import phases, which this engine does not have.
                let phase = first == Some(true)
                    && match tok {
                        Tok::Word(word) => word != "from",
                        Tok::Star | Tok::LBrace => true,
                        _ => false,
                    };
                let depth = match tok {
                    Tok::LBrace => Some(depth + 1),
                    Tok::RBrace => depth.checked_sub(1),
                    // A string is an export name, inside braces or after `as`.
                    _ => Some(depth),
                };
                if let (Some(depth), false) = (depth, phase) {
                    self.state = State::Clause {
                        kind,
                        depth,
                        from: depth == 0 && tok == Tok::Word("from"),
                        first: None,
                    };
                }
            }
            // Rule 8.
            (State::Spec(site), Tok::Word("with" | "assert")) => self.state = State::With(site),
            (State::With(site), Tok::LBrace) => self.out.imports[site].attributes = true,
            (State::MetaDot(at), Tok::Prop("meta")) => self.state = State::Meta(at),
            (State::Meta(at), Tok::Dot | Tok::LBracket) => self.out.metas.push(at),
            (State::Call { import, paren }, Tok::Text(text)) => {
                self.state = State::Arg {
                    import,
                    paren,
                    at: self.locator.at(start),
                    raw: cook(text),
                };
            }
            (State::Arg { at, raw, paren, .. }, Tok::RParen | Tok::Comma) => {
                let site = self.site(ImportKind::Dynamic, raw, at);
                self.mark_call(site, paren, tok == Tok::RParen);
            }
            (State::Call { import, paren } | State::Arg { import, paren, .. }, _) => {
                let site = self.site(ImportKind::DynamicNonLiteral, String::new(), import);
                // An empty `import()` is closed by this very token.
                let closed = tok == Tok::RParen && self.open.len() <= paren;
                self.mark_call(site, paren, closed);
                if !closed {
                    self.fresh(tok, start);
                }
            }
            // After a line end the braces are a block: the `import(…)` before them is a
            // statement that a semicolon is inserted after.
            (State::Closed(site), Tok::LBrace) if !self.line_break => self.dropped.push(site),
            // Rule 11.
            (State::Eval(at), Tok::LParen) => self.out.evals.push(at),
            _ => self.fresh(tok, start),
        }
    }

    /// A token that continues nothing: it may start a rule.
    fn fresh(&mut self, tok: Tok<'a>, start: usize) {
        match tok {
            Tok::Word("import") => self.state = State::Import(self.locator.at(start)),
            Tok::Word("export") => self.state = State::Export,
            Tok::Word("eval" | "Function") if self.prev_word != "function" => {
                self.state = State::Eval(self.locator.at(start));
            }
            _ => {}
        }
    }
}

/// The offset of `needle` in `hay`.
fn find(hay: &[u8], needle: [u8; 2]) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = hay[from..].iter().position(|byte| *byte == needle[0]) {
        if hay.get(from + at + 1) == Some(&needle[1]) {
            return Some(from + at);
        }
        from += at + 1;
    }
    None
}

/// The length of the string literal `rest` starts with, quotes included; `None` when its line
/// or the file ends first.
fn string_len(rest: &[u8]) -> Option<usize> {
    let quote = rest[0];
    let mut at = 1;
    loop {
        match *rest.get(at)? {
            b'\\' => {
                // An escape, a line continuation included (`\` CR LF is one).
                at += if rest.get(at + 1) == Some(&b'\r') && rest.get(at + 2) == Some(&b'\n') {
                    3
                } else {
                    2
                };
            }
            b'\n' | b'\r' => return None,
            byte if byte == quote => return Some(at + 1),
            _ => at += 1,
        }
    }
}

/// The length of the regular-expression literal `rest` starts with, flags included; the
/// offset the scan stopped at when its line or the file ends first (the `/` is then a
/// division). With `to_bracket` the scan also stops, without an end, at the first square
/// bracket.
fn regex_len(rest: &[u8], to_bracket: bool) -> Result<usize, usize> {
    let mut at = 1;
    let mut class = false;
    loop {
        match rest.get(at..).unwrap_or_default() {
            []
            | [b'\n' | b'\r', ..]
            | [0xE2, 0x80, 0xA8 | 0xA9, ..]
            | [b'\\', b'\n' | b'\r', ..] => return Err(at.min(rest.len())),
            [b'\\', ..] => at += 2,
            [b'[' | b']', ..] if to_bracket => return Err(at),
            [b'[', ..] => {
                class = true;
                at += 1;
            }
            [b']', ..] => {
                class = false;
                at += 1;
            }
            [b'/', ..] if !class => break,
            _ => at += 1,
        }
    }
    at += 1;
    while rest
        .get(at)
        .is_some_and(|byte| is_word_byte(*byte) && *byte != b'\\')
    {
        at += 1;
    }
    Ok(at)
}

/// The length of the numeric literal `rest` starts with.
fn number_len(rest: &[u8]) -> usize {
    let radix = matches!(rest, [b'0', b'x' | b'X' | b'b' | b'B' | b'o' | b'O', ..]);
    let mut at = 1;
    let mut dotted = rest[0] == b'.';
    while let Some(byte) = rest.get(at) {
        let sign = matches!(byte, b'+' | b'-') && !radix && matches!(rest[at - 1], b'e' | b'E');
        // The first `.` belongs to the number, a second one does not (`1.5.toFixed()`).
        let dot = *byte == b'.' && !dotted && !radix;
        dotted |= dot;
        if byte.is_ascii_alphanumeric() || *byte == b'_' || sign || dot {
            at += 1;
        } else {
            break;
        }
    }
    at
}
