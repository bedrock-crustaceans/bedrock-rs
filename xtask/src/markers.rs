//! A block of generated lines between two marker comment lines of an otherwise hand-written
//! file: what `cargo xtask addon-features` and `cargo xtask protocol` rewrite in a `Cargo.toml`.

/// The marker line a text lacks.
#[derive(Debug, PartialEq, Eq)]
pub struct MissingMarker<'m>(pub &'m str);

/// `text` with the lines between the `begin` and `end` marker lines replaced by `body`. A marker
/// is matched up to the end of its line, and the `end` line is looked for after the `begin` line.
///
/// # Errors
///
/// Returns the marker that `text` has no line of.
pub fn splice<'m>(
    text: &str,
    begin: &'m str,
    end: &'m str,
    body: &str,
) -> Result<String, MissingMarker<'m>> {
    let begin_line = format!("{begin}\n");
    let start = text.find(&begin_line).ok_or(MissingMarker(begin))? + begin_line.len();
    let length = text[start..].find(&format!("{end}\n"));
    let stop = start + length.ok_or(MissingMarker(end))?;
    Ok(format!("{}{body}{}", &text[..start], &text[stop..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "above\n# begin\nold\nolder\n# end\nbelow\n";

    #[test]
    fn the_lines_between_the_markers_are_replaced_and_the_rest_is_kept() {
        assert_eq!(
            splice(TEXT, "# begin", "# end", "new\n").unwrap(),
            "above\n# begin\nnew\n# end\nbelow\n"
        );
        assert_eq!(
            splice(TEXT, "# begin", "# end", "").unwrap(),
            "above\n# begin\n# end\nbelow\n"
        );
    }

    #[test]
    fn a_text_without_a_marker_line_names_the_marker_it_lacks() {
        assert_eq!(
            splice(TEXT, "# start", "# end", "new\n"),
            Err(MissingMarker("# start"))
        );
        assert_eq!(
            splice(TEXT, "# begin", "# stop", "new\n"),
            Err(MissingMarker("# stop"))
        );
    }

    #[test]
    fn a_marker_ends_its_line_and_the_end_follows_the_begin() {
        // `# end` before the begin line does not close the block, and `# begin here` is not it.
        let text = "# end\n# begin here\n# begin\nold\n# end\n";
        assert_eq!(
            splice(text, "# begin", "# end", "new\n").unwrap(),
            "# end\n# begin here\n# begin\nnew\n# end\n"
        );
        assert_eq!(
            splice("# begin\nold\n", "# begin", "# end", ""),
            Err(MissingMarker("# end"))
        );
    }
}
