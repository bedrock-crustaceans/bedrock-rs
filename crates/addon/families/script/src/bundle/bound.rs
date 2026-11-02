//! The most stack a recursive-descent parser can need for a script file, bounded from the
//! file's bytes alone: the gate that keeps the parser of the `script-syntax` feature off a
//! file it could overflow the stack on.
//!
//! The bound does not use the import lexer's reading of the file. Where the lexical grammar
//! needs the parser to say what a byte is, it follows every reading at once:
//!
//! - a `/` starts a regular expression, or divides, or both, unless the token before it
//!   leaves one reading (see [`Reading::slash`]);
//! - a regular expression's `/` inside a class does not end it, and a class is open or not
//!   depending on where the expression began: both are followed;
//! - a string or a regular expression that its line ends in is an error, after which the
//!   text is read on as code;
//! - a `}` ends a template substitution or closes a brace, as the bracket it closes says.
//!
//! A string, a template, a comment and a `${` start where the grammar says, in every
//! reading that is in code there. Each reading keeps what a parser has on its stack: the
//! open brackets and, per bracket, the operators and statements that wait for their right
//! side. Readings that meet in the same lexical state are joined into one that needs at
//! least the stack of either, so the scan keeps one reading per state and is linear in the
//! file. The bound is the highest need of any reading at any byte.
// The gate must not depend on the import lexer's reading, because a wrong reading there
// would let a file past that the parser then overflows on. The weights below are those of
// `oxc_parser` 0.153 in an unoptimised build with full debug info, the largest frames it has.

/// The parser is given a file only when the bound is at most this many bytes: a quarter of
/// the megabyte a world load must fit in.
// The parser recurses and has no depth limit of its own; an overflow aborts the process.
pub(super) const STACK_BUDGET: u64 = 256 * 1024;

/// The stack one open bracket or template substitution costs, in bytes.
const FRAME: u64 = 4608;
/// The stack one pending right-binding operator or expression keyword costs.
const CHAIN: u64 = 2048;
/// The stack one pending unbraced statement or label costs.
const STATEMENT: u64 = 2048;
/// How many pending operators the `extends` of a class counts as.
const HERITAGE: u32 = 2;
/// The stack one step up the binary precedences costs.
const LADDER: u64 = 1400;

/// The binary precedence levels, lowest first, one bit each.
mod level {
    pub(super) const COALESCE: u16 = 1 << 0;
    pub(super) const OR: u16 = 1 << 1;
    pub(super) const AND: u16 = 1 << 2;
    pub(super) const BIT_OR: u16 = 1 << 3;
    pub(super) const BIT_XOR: u16 = 1 << 4;
    pub(super) const BIT_AND: u16 = 1 << 5;
    pub(super) const EQUALITY: u16 = 1 << 6;
    pub(super) const RELATIONAL: u16 = 1 << 7;
    pub(super) const SHIFT: u16 = 1 << 8;
    pub(super) const ADDITIVE: u16 = 1 << 9;
    pub(super) const MULTIPLICATIVE: u16 = 1 << 10;
}

/// What an open bracket may be.
mod kind {
    /// A bracket that is no template substitution: its `}` leaves the reading in code.
    pub(super) const PLAIN: u8 = 1;
    /// `${`: its `}` goes on with the template's text.
    pub(super) const SUBSTITUTION: u8 = 2;
    /// The `(` of `if`, `while`, `for` or `with`: a statement follows its `)`.
    pub(super) const HEAD: u8 = 4;
    /// The file itself, which nothing closes.
    pub(super) const FILE: u8 = PLAIN | HEAD;
}

/// What a reading knows of the token before the current byte. The first eight say what may
/// be so and are joined by "or"; the last two say what is so and are joined by "and".
mod flag {
    /// The token may end an expression: a `/` after it may divide.
    pub(super) const OPERAND: u16 = 1 << 0;
    /// The token may be one an expression follows: a `/` after it may start a regular
    /// expression.
    pub(super) const OPERATOR: u16 = 1 << 1;
    /// A line may have ended since the token.
    pub(super) const LINE: u16 = 1 << 2;
    /// The token may be the `.` of a member access: a word after it is then a property name.
    pub(super) const DOT: u16 = 1 << 3;
    /// The token may be something else than that `.`.
    pub(super) const NAME: u16 = 1 << 4;
    /// The token may be `if`, `while`, `for`, `with` or `await`: a `(` after it is a head's.
    pub(super) const HEAD: u16 = 1 << 5;
    /// A statement may begin after the token.
    pub(super) const STATEMENT: u16 = 1 << 6;
    /// The token may be a name at the start of a statement: a `:` after it is a label's.
    pub(super) const LABEL: u16 = 1 << 7;
    /// The token is a `;`: the unbraced statements before it have ended, unless `else` or
    /// `while` is the next token.
    pub(super) const END: u16 = 1 << 8;
    /// The token is a `}`.
    pub(super) const BRACE: u16 = 1 << 9;

    /// The flags that say what is so.
    pub(super) const CERTAIN: u16 = END | BRACE;
    /// After a token an expression follows.
    pub(super) const AFTER_OPERATOR: u16 = OPERATOR | NAME;
    /// After a token that ends an expression.
    pub(super) const AFTER_OPERAND: u16 = OPERAND | NAME;
}

/// What waits inside one open bracket (or in the file, outside every bracket).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct Level {
    /// Unbraced statements and labels.
    statements: u32,
    /// Right-binding operators and expression keywords.
    operators: u32,
    /// The binary precedence levels in use.
    ladder: u16,
    /// What the bracket may be (`kind`).
    kind: u8,
}

impl Level {
    fn bytes(self) -> u64 {
        u64::from(self.statements) * STATEMENT
            + u64::from(self.operators) * CHAIN
            + u64::from(self.ladder.count_ones()) * LADDER
    }
}

/// One reading of the file up to the current byte: what a parser that reads it so has on its
/// stack.
#[derive(Clone, Debug)]
struct Reading {
    /// The file, then the open brackets, the innermost last. Never empty.
    levels: Vec<Level>,
    /// The stack the levels need, in bytes.
    cost: u64,
    /// `flag`.
    flags: u16,
}

impl Reading {
    fn new() -> Self {
        Self {
            levels: vec![Level {
                kind: kind::FILE,
                ..Level::default()
            }],
            cost: 0,
            flags: flag::AFTER_OPERATOR | flag::STATEMENT,
        }
    }

    /// Changes the innermost level and keeps the cost in step.
    fn change(&mut self, change: impl FnOnce(&mut Level)) {
        let Some(top) = self.levels.last_mut() else {
            return;
        };
        let before = top.bytes();
        change(top);
        self.cost = self.cost + top.bytes() - before;
    }

    fn operator(&mut self) {
        self.change(|top| top.operators += 1);
    }

    fn statement(&mut self) {
        self.change(|top| top.statements += 1);
    }

    fn binary(&mut self, level: u16) {
        self.change(|top| top.ladder |= level);
    }

    /// Everything that waits in the innermost level has returned.
    fn clear(&mut self) {
        self.change(|top| {
            top.statements = 0;
            top.operators = 0;
            top.ladder = 0;
        });
    }

    /// The expressions of the innermost level have returned.
    fn clear_expressions(&mut self) {
        self.change(|top| {
            top.operators = 0;
            top.ladder = 0;
        });
    }

    fn open(&mut self, kind: u8) {
        self.levels.push(Level {
            kind,
            ..Level::default()
        });
        self.cost += FRAME;
    }

    /// A closing bracket: the innermost level is gone, and what it was is returned. Outside
    /// every bracket nothing can take the token, so whatever waited there has returned.
    // A frame of a recursive-descent parser stays on the stack only while the tokens are
    // taken by it or below it: one that cannot take a closing bracket has returned (or the
    // parse has stopped) when the bracket is read.
    fn close(&mut self) -> u8 {
        if self.levels.len() > 1 {
            let closed = self.levels.pop().unwrap_or_default();
            self.cost -= FRAME + closed.bytes();
            closed.kind
        } else {
            self.clear();
            kind::FILE
        }
    }

    /// Called at every token: settles whether the statements before a `;` have ended.
    /// `goes_on` for `else` and `while`.
    fn token(&mut self, goes_on: bool) {
        if self.flags & flag::END != 0 {
            self.flags &= !flag::END;
            if !goes_on {
                // `if (a) x; else y` and `do x; while (c)` go on past the `;`; nothing else
                // does.
                self.change(|top| top.statements = 0);
            }
        }
    }

    /// The token before may have ended an expression, and nothing else.
    fn after_operand(&self) -> bool {
        self.flags & (flag::OPERAND | flag::OPERATOR | flag::LINE) == flag::OPERAND
    }

    /// Whether a `/` that is no comment may divide and may start a regular expression.
    // The parser asks for a regular expression only where an expression starts, and for a
    // division only after an operand. After a token that ends an expression on the same
    // line no expression can start (a semicolon is inserted only at a line end, a `}` or the
    // end of the file), and after a token that an expression follows none has ended.
    fn slash(&self) -> (bool, bool) {
        let divides = self.flags & flag::OPERAND != 0;
        let regex = self.flags & (flag::OPERATOR | flag::LINE) != 0;
        (divides, regex || !divides)
    }

    /// Makes `self` a reading that needs at least the stack of `self` and of `other`,
    /// whatever follows.
    // The levels are matched from the innermost outwards and each takes the larger counts:
    // every token changes the innermost level of both in the same way, so the joined
    // reading stays above both. Where `other` has fewer brackets open, the brackets of
    // `self` beyond them may also be none (`other` is then outside every bracket).
    fn join(&mut self, mut other: Self) {
        if other.levels.len() > self.levels.len() {
            std::mem::swap(&mut self.levels, &mut other.levels);
        }
        let beyond = self.levels.len() - other.levels.len();
        let (outer, matched) = self.levels.split_at_mut(beyond);
        for level in outer {
            level.kind |= kind::FILE;
        }
        for (level, with) in matched.iter_mut().zip(&other.levels) {
            level.statements = level.statements.max(with.statements);
            level.operators = level.operators.max(with.operators);
            level.ladder |= with.ladder;
            level.kind |= with.kind;
        }
        self.cost = self.levels.iter().map(|level| level.bytes()).sum::<u64>()
            + FRAME * (self.levels.len() as u64 - 1);
        let certain = self.flags & other.flags & flag::CERTAIN;
        self.flags = ((self.flags | other.flags) & !flag::CERTAIN) | certain;
    }
}

/// Adds `reading` to the readings of one lexical state.
fn merge(slot: &mut Option<Reading>, reading: Reading) {
    match slot {
        Some(held) => held.join(reading),
        None => *slot = Some(reading),
    }
}

/// What a word is to the parser's recursion, when it is no property name.
#[derive(Clone, Copy)]
struct Word {
    count: Count,
    ends: Ends,
    /// The word is `if`, `while`, `for`, `with` or `await`.
    head: bool,
    /// A statement follows the word: `else`, `do`.
    statement: bool,
}

/// Whether a word says that the statement before it has ended.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ends {
    Nothing,
    /// The word can only begin a statement (or a declaration): straight after a `}` or an
    /// operand, the statement before it has ended.
    AfterOperand,
    /// `class`: straight after a `}` it begins a declaration. (After an operand it may be a
    /// decorated class expression, `x = @d class {}`.)
    AfterBrace,
    /// `case` and `default`: a clause of a switch, which no statement of the clause before
    /// reaches into.
    Always,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Count {
    Nothing,
    Statement,
    Operator,
    Heritage,
    Binary(u16),
}

/// The reserved and contextual words; `None` for a name (`this`, `super`, `null`, `true` and
/// `false` among them, which end an expression as a name does).
fn word(word: &[u8]) -> Option<Word> {
    let plain = Word {
        count: Count::Nothing,
        ends: Ends::Nothing,
        head: false,
        statement: false,
    };
    let statement = Word {
        count: Count::Statement,
        ends: Ends::AfterOperand,
        ..plain
    };
    let begins = Word {
        ends: Ends::AfterOperand,
        ..plain
    };
    let operator = Word {
        count: Count::Operator,
        ..plain
    };
    Some(match word {
        b"if" | b"for" | b"with" => Word {
            head: true,
            ..statement
        },
        // `do x; while (c)`: the `do` still waits when its `while` is read.
        b"while" => Word {
            head: true,
            ends: Ends::Nothing,
            ..statement
        },
        b"do" => Word {
            statement: true,
            ..statement
        },
        b"else" => Word {
            statement: true,
            ends: Ends::Nothing,
            ..statement
        },
        b"switch" | b"try" => statement,
        b"catch" | b"finally" => Word {
            ends: Ends::Nothing,
            ..statement
        },
        b"return" | b"throw" => Word {
            ends: Ends::AfterOperand,
            ..operator
        },
        b"case" => Word {
            ends: Ends::Always,
            ..operator
        },
        b"default" => Word {
            ends: Ends::Always,
            ..plain
        },
        b"const" | b"var" | b"break" | b"continue" | b"debugger" | b"export" | b"import"
        | b"function" => begins,
        b"await" => Word {
            head: true,
            ..operator
        },
        b"delete" | b"typeof" | b"void" | b"new" | b"yield" => operator,
        // A class and the expression it extends: two of the parser's larger frames.
        b"extends" => Word {
            count: Count::Heritage,
            ..plain
        },
        b"class" => Word {
            ends: Ends::AfterBrace,
            ..plain
        },
        b"in" | b"instanceof" => Word {
            count: Count::Binary(level::RELATIONAL),
            ..plain
        },
        b"enum" | b"let" | b"static" | b"async" | b"of" | b"get" | b"set" | b"from" | b"as"
        | b"using" | b"accessor" | b"implements" | b"interface" | b"package" | b"private"
        | b"protected" | b"public" => plain,
        _ => return None,
    })
}

fn is_ascii_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

/// A byte that may be part of a name without being an ASCII letter: a character outside
/// ASCII (which may also be white space) or the `\` of an escape.
fn is_loose(byte: u8) -> bool {
    byte >= 0x80 || byte == b'\\'
}

/// The readings of the file up to the current byte, one per lexical state.
#[derive(Default)]
struct Scan {
    code: Option<Reading>,
    /// In a regular expression.
    regex: Option<Reading>,
    /// A reading in a regular expression may be inside a class.
    regex_class: bool,
    /// In a `"…"` string.
    double: Option<Reading>,
    /// In a `'…'` string.
    single: Option<Reading>,
    /// In the text of a template.
    template: Option<Reading>,
    /// In a `//` comment.
    line: Option<Reading>,
    /// Before the `*` of a `/*`.
    block_star: Option<Reading>,
    /// Before the first byte of a block comment's text, which ends no comment.
    block_first: Option<Reading>,
    /// In a block comment.
    block: Option<Reading>,
}

/// The most stack a recursive-descent parser can need for `source`, in bytes, whatever its
/// reading of the file; `None` when that is more than `budget`.
pub(super) fn parser_stack(source: &str, budget: u64) -> Option<u64> {
    let bytes = source.as_bytes();
    let mut at = if bytes.starts_with(b"\xEF\xBB\xBF") {
        3
    } else {
        0
    };
    let mut scan = Scan {
        code: Some(Reading::new()),
        ..Scan::default()
    };
    if bytes[at..].starts_with(b"#!") {
        // A hashbang line is a comment.
        scan.line.clone_from(&scan.code);
    }
    let mut max = 0;
    // The backslashes straight before the current byte: an odd count escapes it.
    let mut backslashes = 0usize;
    let mut before_escaped = false;
    while at < bytes.len() {
        let escaped = backslashes % 2 == 1;
        let arrived = scan.text_states(bytes, at, escaped, before_escaped);
        let mut code = arrived;
        if let Some(reading) = scan.code.take() {
            scan.code_byte(reading, bytes, at, &mut code);
        }
        if let Some(reading) = &code {
            max = max.max(reading.cost);
            if max > budget {
                return None;
            }
        }
        scan.code = code;
        before_escaped = escaped;
        backslashes = if bytes[at] == b'\\' {
            backslashes + 1
        } else {
            0
        };
        at += 1;
    }
    Some(max)
}

/// `rest` starts with a line terminator.
fn is_line_end(rest: &[u8]) -> bool {
    matches!(rest, [b'\n' | b'\r', ..] | [0xE2, 0x80, 0xA8 | 0xA9, ..])
}

impl Scan {
    /// The byte at `at` for the readings that are in a comment, a string, a template or a
    /// regular expression: returns those it brings back to code.
    fn text_states(
        &mut self,
        bytes: &[u8],
        at: usize,
        escaped: bool,
        before_escaped: bool,
    ) -> Option<Reading> {
        let byte = bytes[at];
        let before = at.checked_sub(1).map_or(0, |at| bytes[at]);
        let line_end = is_line_end(&bytes[at..]);
        let mut code = None;
        if line_end && let Some(mut reading) = self.line.take() {
            reading.flags |= flag::LINE;
            merge(&mut code, reading);
        }
        if byte == b'/'
            && before == b'*'
            && let Some(mut reading) = self.block.take()
        {
            // A comment may hold a line end, and is one then.
            reading.flags |= flag::LINE;
            merge(&mut code, reading);
        }
        if let Some(reading) = self.block_first.take() {
            merge(&mut self.block, reading);
        }
        self.block_first = self.block_star.take();
        for (slot, quote) in [(&mut self.double, b'"'), (&mut self.single, b'\'')] {
            if escaped || slot.is_none() {
                continue;
            }
            // `\` CR LF is one line continuation.
            let continued = byte == b'\n' && before == b'\r' && before_escaped;
            if byte == quote {
                if let Some(mut reading) = slot.take() {
                    reading.flags = flag::AFTER_OPERAND;
                    merge(&mut code, reading);
                }
            } else if matches!(byte, b'\n' | b'\r') && !continued {
                // An unterminated string: what follows is read as code, whatever the
                // parser makes of the error.
                if let Some(mut reading) = slot.take() {
                    reading.flags = flag::AFTER_OPERAND | flag::OPERATOR | flag::LINE;
                    merge(&mut code, reading);
                }
            }
        }
        if !escaped && self.template.is_some() {
            if byte == b'`' {
                if let Some(mut reading) = self.template.take() {
                    reading.flags = flag::AFTER_OPERAND;
                    merge(&mut code, reading);
                }
            } else if byte == b'{'
                && before == b'$'
                && !before_escaped
                && let Some(mut reading) = self.template.take()
            {
                reading.open(kind::SUBSTITUTION);
                reading.flags = flag::AFTER_OPERATOR;
                merge(&mut code, reading);
            }
        }
        if self.regex.is_some() {
            if line_end {
                // An unterminated regular expression ends with its line.
                if let Some(mut reading) = self.regex.take() {
                    reading.flags = flag::AFTER_OPERAND | flag::LINE;
                    merge(&mut code, reading);
                }
                self.regex_class = false;
            } else if !escaped {
                match byte {
                    b'[' => self.regex_class = true,
                    b']' => self.regex_class = false,
                    b'/' => {
                        // Inside a class the `/` ends nothing; a reading that began after
                        // the class opened is in none.
                        let ended = if self.regex_class {
                            self.regex.clone()
                        } else {
                            self.regex.take()
                        };
                        if let Some(mut reading) = ended {
                            reading.flags = flag::AFTER_OPERAND;
                            merge(&mut code, reading);
                        }
                    }
                    _ => {}
                }
            }
        }
        code
    }

    /// The byte at `at` for the reading that is in code.
    #[allow(
        clippy::too_many_lines,
        reason = "one arm per kind of byte, each a few lines: what a token is to the parser's stack in one place"
    )]
    fn code_byte(
        &mut self,
        mut reading: Reading,
        bytes: &[u8],
        at: usize,
        code: &mut Option<Reading>,
    ) {
        let byte = bytes[at];
        // The neighbours: a byte that brings a reading to code (a quote, a `/`, a line end, a
        // `{` or a backquote) is none of the operator bytes asked for below, so a neighbour
        // that is one was read in code by this reading.
        let before = at.checked_sub(1).map_or(0, |at| bytes[at]);
        let before2 = at.checked_sub(2).map_or(0, |at| bytes[at]);
        let after = bytes.get(at + 1).copied().unwrap_or(0);
        let after2 = bytes.get(at + 2).copied().unwrap_or(0);
        match byte {
            b' ' | b'\t' | 0x0B | 0x0C => {}
            b'\n' | b'\r' => reading.flags |= flag::LINE,
            _ if is_ascii_word(byte) => {
                if !is_ascii_word(before) {
                    word_token(&mut reading, bytes, at);
                }
            }
            _ if is_loose(byte) => {
                if is_line_end(&bytes[at..]) {
                    reading.flags |= flag::LINE;
                } else {
                    // Part of a name, or white space.
                    reading.flags = (reading.flags | flag::OPERAND | flag::NAME) & !flag::BRACE;
                }
            }
            b'"' => {
                reading.token(false);
                merge(&mut self.double, reading);
                return;
            }
            b'\'' => {
                reading.token(false);
                merge(&mut self.single, reading);
                return;
            }
            b'`' => {
                reading.token(false);
                merge(&mut self.template, reading);
                return;
            }
            // A comment is no token: the reading comes back as it is.
            b'/' if after == b'/' => {
                merge(&mut self.line, reading);
                return;
            }
            b'/' if after == b'*' => {
                merge(&mut self.block_star, reading);
                return;
            }
            b'/' => {
                reading.token(false);
                let (divides, regex) = reading.slash();
                if regex {
                    if self.regex.is_none() {
                        self.regex_class = false;
                    }
                    if divides {
                        merge(&mut self.regex, reading.clone());
                    } else {
                        merge(&mut self.regex, reading);
                        return;
                    }
                }
                // `/=` is counted at its `=`.
                if after != b'=' {
                    reading.binary(level::MULTIPLICATIVE);
                }
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'(' | b'[' | b'{' => {
                reading.token(false);
                let head = byte == b'(' && reading.flags & flag::HEAD != 0;
                reading.open(if head {
                    kind::PLAIN | kind::HEAD
                } else {
                    kind::PLAIN
                });
                reading.flags = flag::AFTER_OPERATOR;
                if byte == b'{' {
                    reading.flags |= flag::STATEMENT;
                }
            }
            b')' | b']' => {
                reading.token(false);
                let closed = reading.close();
                reading.flags = flag::AFTER_OPERAND;
                if byte == b')' {
                    reading.flags |= flag::STATEMENT;
                    if closed & kind::HEAD != 0 {
                        reading.flags |= flag::OPERATOR;
                    }
                }
            }
            b'}' => {
                reading.token(false);
                // The innermost open bracket is the one a `}` closes.
                let closed = reading.close();
                reading.flags =
                    flag::AFTER_OPERAND | flag::OPERATOR | flag::STATEMENT | flag::BRACE;
                if closed & kind::SUBSTITUTION != 0 {
                    if closed & kind::PLAIN == 0 {
                        merge(&mut self.template, reading);
                        return;
                    }
                    merge(&mut self.template, reading.clone());
                }
            }
            b';' => {
                reading.token(false);
                reading.clear_expressions();
                reading.flags = flag::AFTER_OPERATOR | flag::STATEMENT | flag::END;
            }
            b',' => {
                reading.token(false);
                // Every right-binding expression is below the comma: a list's element or a
                // sequence's term has ended.
                reading.clear_expressions();
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'.' => {
                reading.token(false);
                if before == b'.' || after == b'.' {
                    // `...`, counted once.
                    if before != b'.' {
                        reading.operator();
                    }
                    reading.flags = flag::OPERATOR | flag::DOT | flag::NAME;
                } else if before.is_ascii_digit() {
                    // The `.` may end a number (`1.`): what follows is then no property.
                    reading.flags = flag::AFTER_OPERAND | flag::OPERATOR | flag::DOT;
                } else {
                    reading.flags = flag::OPERATOR | flag::DOT;
                }
            }
            b'?' => {
                reading.token(false);
                if before == b'?' || after == b'?' {
                    reading.binary(level::COALESCE);
                } else if after != b'.' || after2.is_ascii_digit() {
                    reading.operator();
                }
                reading.flags = flag::AFTER_OPERATOR;
            }
            b':' => {
                reading.token(false);
                if reading.flags & flag::LABEL != 0 {
                    reading.statement();
                } else {
                    reading.operator();
                }
                reading.flags = flag::AFTER_OPERATOR | flag::STATEMENT;
            }
            b'~' => {
                reading.token(false);
                reading.operator();
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'!' => {
                reading.token(false);
                // `!=` and `!==` are counted at their `=`.
                if after != b'=' {
                    reading.operator();
                }
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'=' => {
                reading.token(false);
                if after == b'>' {
                    reading.operator();
                } else if after == b'=' || matches!(before, b'=' | b'!') {
                    reading.binary(level::EQUALITY);
                } else if matches!(before, b'<' | b'>') && before2 != before {
                    reading.binary(level::RELATIONAL);
                } else {
                    reading.operator();
                }
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'<' | b'>' => {
                reading.token(false);
                if byte == b'>' && before == b'=' {
                    // The `>` of `=>`, counted at its `=`.
                } else if before == byte || after == byte {
                    reading.binary(level::SHIFT);
                } else {
                    reading.binary(level::RELATIONAL);
                }
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'&' | b'|' => {
                reading.token(false);
                let doubled = before == byte || after == byte;
                reading.binary(match (byte, doubled) {
                    (b'&', true) => level::AND,
                    (b'&', false) => level::BIT_AND,
                    (_, true) => level::OR,
                    (_, false) => level::BIT_OR,
                });
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'^' => {
                reading.token(false);
                reading.binary(level::BIT_XOR);
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'%' => {
                reading.token(false);
                reading.binary(level::MULTIPLICATIVE);
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'*' => {
                reading.token(false);
                if after == b'*' {
                    // `**` binds to its right.
                    reading.operator();
                } else if before != b'*' {
                    reading.binary(level::MULTIPLICATIVE);
                }
                reading.flags = flag::AFTER_OPERATOR;
            }
            b'+' | b'-' => {
                reading.token(false);
                if before == byte || after == byte {
                    // `++` and `--`, prefix or postfix: each byte as a prefix operator.
                    reading.operator();
                    reading.flags = flag::AFTER_OPERAND | flag::OPERATOR;
                } else {
                    if reading.after_operand() {
                        reading.binary(level::ADDITIVE);
                    } else {
                        reading.operator();
                    }
                    reading.flags = flag::AFTER_OPERATOR;
                }
            }
            b'@' => {
                reading.token(false);
                // A decorator's expression may start with a decorator (of a class
                // expression); after an operand the `@` is the next of a list.
                if reading.flags & (flag::OPERAND | flag::OPERATOR) != flag::OPERAND {
                    reading.operator();
                }
                reading.flags = flag::AFTER_OPERATOR;
            }
            // `#` of a private name (the name follows as a word) and bytes that are no
            // token.
            _ => {}
        }
        merge(code, reading);
    }
}

/// The word that starts at `at`: the ASCII letters, digits, `_` and `$` up to the next other
/// byte.
fn word_token(reading: &mut Reading, bytes: &[u8], at: usize) {
    {
        let end = bytes[at..]
            .iter()
            .position(|byte| !is_ascii_word(*byte))
            .map_or(bytes.len(), |len| at + len);
        let text = &bytes[at..end];
        // The word may be part of a longer name: a character outside ASCII or an escape
        // touches it.
        let part = at.checked_sub(1).is_some_and(|at| is_loose(bytes[at]))
            || bytes.get(end).is_some_and(|byte| is_loose(*byte));
        let known = word(text);
        reading.token(matches!(text, b"else" | b"while"));
        let may_be_property = reading.flags & flag::DOT != 0;
        let may_be_word = reading.flags & flag::NAME != 0;
        let statement_may_begin = reading.flags & flag::STATEMENT != 0
            || reading.flags & (flag::OPERAND | flag::LINE) == flag::OPERAND | flag::LINE;
        if text[0].is_ascii_digit() {
            reading.flags = flag::AFTER_OPERAND;
            return;
        }
        let Some(known) = known.filter(|_| may_be_word) else {
            // A name, or a property name.
            reading.flags = flag::AFTER_OPERAND;
            if may_be_word && statement_may_begin {
                reading.flags |= flag::LABEL;
            }
            return;
        };
        let certain = !may_be_property && !part;
        if certain {
            // Straight after a `}` or an operand, a word that can only begin a statement
            // says that the statement before it has ended.
            let ended = reading.flags & flag::BRACE != 0
                || reading.flags & (flag::OPERAND | flag::OPERATOR) == flag::OPERAND;
            let after_brace = reading.flags & flag::BRACE != 0;
            let clears = match known.ends {
                Ends::Nothing => false,
                Ends::AfterOperand => ended,
                Ends::AfterBrace => after_brace,
                Ends::Always => true,
            };
            if clears {
                reading.clear();
            }
        }
        match known.count {
            Count::Nothing => {}
            Count::Statement => reading.statement(),
            Count::Operator => reading.operator(),
            Count::Heritage => reading.change(|top| top.operators += HERITAGE),
            Count::Binary(level) => reading.binary(level),
        }
        // An expression may follow the word, and the word may be a name that ends one (`of`,
        // `async`, `get`; and whatever the parser lets pass as one).
        let mut flags = flag::AFTER_OPERAND | flag::OPERATOR;
        if !certain && statement_may_begin {
            flags |= flag::LABEL;
        }
        if known.head {
            flags |= flag::HEAD;
        }
        if known.statement {
            flags |= flag::STATEMENT;
        }
        reading.flags = flags;
    }
}
