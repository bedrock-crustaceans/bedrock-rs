//! Writing a function back as `.mcfunction` text (its command lines only) and a tick list back
//! as JSON.

use facet_value::{VArray, VObject};

use super::parse::VALUES;
use super::{Function, FunctionLine, TickList};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value, WriteStyle, write_json};

impl Function {
    /// Writes the command lines to `out`, each as written, ended by a line feed; the last one
    /// only when the style asks for a trailing newline. Comments, blank lines and dropped
    /// lines are not part of a function and are not written. Nothing is appended when a line
    /// cannot be written.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        let unwritable = self.commands.iter().enumerate().position(|(index, line)| {
            // Read back, a byte order mark at the start of the file is stripped.
            !reads_back(line) || (index == 0 && line.raw.starts_with('\u{feff}'))
        });
        if let Some(index) = unwritable {
            let at = JsonPath::new().child("commands").index(index);
            return Err(EncodeError::NotRepresentable {
                at,
                what: Unrepresentable::Unsupported,
            });
        }
        for (index, line) in self.commands.iter().enumerate() {
            if index > 0 {
                out.push(b'\n');
            }
            out.extend_from_slice(line.raw.as_bytes());
        }
        if style.trailing_newline && !self.commands.is_empty() {
            out.push(b'\n');
        }
        Ok(())
    }
}

/// Whether `line`, written, is read back as the same one command: it holds no line feed, does
/// not end in the carriage return of a line ending, and is neither blank nor a comment.
fn reads_back(line: &FunctionLine) -> bool {
    let command = line.command();
    !(line.raw.contains('\n')
        || line.raw.ends_with('\r')
        || command.is_empty()
        || command.starts_with('#'))
}

impl TickList {
    /// Writes the list as JSON text to `out`: `values`, then the members vanilla does not read,
    /// in their order. The entries' command versions are the pack's and are not written.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        let names: Vec<Value> = self
            .values
            .iter()
            .map(|entry| Value::from(entry.function.as_str()))
            .collect();
        let mut root = VObject::new();
        root.insert(VALUES, VArray::from(names));
        for (key, value) in self.extra.iter() {
            // `values` is the list's own member.
            if key != VALUES {
                root.insert(key, value.clone());
            }
        }
        write_json(&Value::from(root), out, style)
    }
}
