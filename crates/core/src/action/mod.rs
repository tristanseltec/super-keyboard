//! Key actions and the `<name>` notation used in layout files.
//!
//! A key in a layout file is written as a short string:
//!
//! | Written as      | Meaning                                                   |
//! |-----------------|-----------------------------------------------------------|
//! | `a`, `->`, `::` | Insert that text.                                         |
//! | `<esc>`         | Press a named key (see [`NamedKey::from_name`]).          |
//! | `<ctrl>`        | Arm a modifier for the next key (see [`Modifier`]).       |
//! | `<layer:sym>`   | Switch to the layer named `sym`.                          |
//!
//! Anything that is not `<identifier>` is plain text, so operators such as `<=>`
//! or `</` need no escaping. Text that looks like a key name (for example `<div>`)
//! is written with the table form `{ text = "<div>" }` instead.

mod modifier;
mod named_key;

pub use modifier::{Modifier, Modifiers};
pub use named_key::NamedKey;

/// What a combining mark is drawn on when it is a label of its own.
const DOTTED_CIRCLE: char = '\u{25cc}';

/// Whether `c` is in Unicode's Combining Diacritical Marks block, where the
/// Vietnamese tone marks are.
pub(crate) fn is_combining_mark(c: char) -> bool {
    ('\u{300}'..='\u{36f}').contains(&c)
}

/// What happens when a key is tapped or held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Insert text. A single character responds to Shift and forms a chord with
    /// Ctrl, Alt or Super; longer text is inserted as written.
    Text(String),
    /// Press a key that does not insert text, such as `<esc>` or `<left>`.
    Key(NamedKey),
    /// Arm a modifier for the next key. Tapping it again locks it; a third tap
    /// releases it.
    Modifier(Modifier),
    /// Switch to the named layer, or back to the first layer if that layer is
    /// already showing.
    Layer(String),
}

/// Why a key could not be parsed from the `<name>` notation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseActionError {
    #[error("empty key")]
    Empty,
    #[error("`<layer:>` needs a layer name")]
    MissingLayerName,
    #[error("unknown key `<{0}>`")]
    UnknownName(String),
}

impl Action {
    /// Parses the short notation used for keys in layout files.
    pub fn parse(spec: &str) -> Result<Self, ParseActionError> {
        let Some(name) = bracketed_name(spec) else {
            if spec.is_empty() {
                return Err(ParseActionError::Empty);
            }
            return Ok(Self::Text(spec.to_owned()));
        };
        if let Some(layer) = name.strip_prefix("layer:") {
            if layer.is_empty() {
                return Err(ParseActionError::MissingLayerName);
            }
            return Ok(Self::Layer(layer.to_owned()));
        }
        if let Some(modifier) = Modifier::from_name(name) {
            return Ok(Self::Modifier(modifier));
        }
        if let Some(key) = NamedKey::from_name(name) {
            return Ok(Self::Key(key));
        }
        Err(ParseActionError::UnknownName(name.to_owned()))
    }

    /// The label shown on a key cap when the layout does not set one. A
    /// combining mark on its own, such as a Vietnamese tone, is drawn on a
    /// dotted circle (◌́) so that it has something to sit on.
    pub fn default_label(&self) -> String {
        match self {
            Self::Text(text) if text.starts_with(is_combining_mark) => {
                format!("{DOTTED_CIRCLE}{text}")
            }
            Self::Text(text) => text.clone(),
            Self::Key(key) => key.label(),
            Self::Modifier(modifier) => modifier.label().to_owned(),
            Self::Layer(layer) => layer.clone(),
        }
    }

    /// Whether holding a key with this action repeats it unless the layout says
    /// otherwise: Backspace, Delete and the cursor keys.
    pub fn repeats_by_default(&self) -> bool {
        matches!(self, Self::Key(key) if key.repeats_by_default())
    }

    /// The single character this action inserts, if it inserts exactly one.
    ///
    /// Single characters respond to Shift and form chords with other modifiers.
    pub fn single_char(&self) -> Option<char> {
        let Self::Text(text) = self else {
            return None;
        };
        let mut chars = text.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        }
    }
}

/// Returns the inner name of `<name>` when `spec` uses the key notation.
fn bracketed_name(spec: &str) -> Option<&str> {
    let inner = spec.strip_prefix('<')?.strip_suffix('>')?;
    let is_name = !inner.is_empty()
        && inner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_'));
    is_name.then_some(inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_combining_mark_is_labelled_on_a_dotted_circle() {
        let label = |text: &str| Action::Text(text.into()).default_label();
        assert_eq!(label("\u{301}"), "\u{25cc}\u{301}");
        assert_eq!(label("\u{323}"), "\u{25cc}\u{323}");
        // Text that only contains a mark is shown as written.
        assert_eq!(label("a\u{301}"), "a\u{301}");
        assert_eq!(label("->"), "->");
        assert_eq!(label(""), "");
    }

    #[test]
    fn plain_text_and_operators_are_text() {
        for spec in ["a", "->", "::", "<", ">", "<>", "<=>", "</", "<=", "{}"] {
            assert_eq!(
                Action::parse(spec),
                Ok(Action::Text(spec.to_owned())),
                "{spec}"
            );
        }
    }

    #[test]
    fn bracketed_names_are_keys_modifiers_and_layers() {
        assert_eq!(Action::parse("<esc>"), Ok(Action::Key(NamedKey::Escape)));
        assert_eq!(Action::parse("<f12>"), Ok(Action::Key(NamedKey::F(12))));
        assert_eq!(
            Action::parse("<ctrl>"),
            Ok(Action::Modifier(Modifier::Ctrl))
        );
        assert_eq!(
            Action::parse("<layer:sym>"),
            Ok(Action::Layer("sym".to_owned()))
        );
    }

    #[test]
    fn unknown_names_and_empty_keys_are_errors() {
        assert_eq!(
            Action::parse("<div>"),
            Err(ParseActionError::UnknownName("div".to_owned()))
        );
        assert!(Action::parse("<f25>").is_err());
        assert_eq!(
            Action::parse("<layer:>"),
            Err(ParseActionError::MissingLayerName)
        );
        assert_eq!(Action::parse(""), Err(ParseActionError::Empty));
    }

    #[test]
    fn editing_and_cursor_keys_repeat_by_default() {
        for spec in ["<bksp>", "<del>", "<left>", "<right>", "<up>", "<down>"] {
            assert!(Action::parse(spec).unwrap().repeats_by_default(), "{spec}");
        }
        for spec in [
            "a",
            "->",
            "<esc>",
            "<enter>",
            "<space>",
            "<ctrl>",
            "<layer:x>",
        ] {
            assert!(!Action::parse(spec).unwrap().repeats_by_default(), "{spec}");
        }
    }

    #[test]
    fn single_char_counts_characters_not_bytes() {
        assert_eq!(Action::Text("é".into()).single_char(), Some('é'));
        assert_eq!(Action::Text("->".into()).single_char(), None);
        assert_eq!(Action::Key(NamedKey::Tab).single_char(), None);
    }
}
