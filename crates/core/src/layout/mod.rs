//! Keyboard layouts: sections of rows, combined into layers, loaded from TOML.
//!
//! ```toml
//! name = "tiny"
//!
//! [[sections]]
//! name = "numbers"
//! title = "Number row"
//! enabled = false            # off unless a profile turns it on
//! rows = [["1", "2", "3"]]
//!
//! [[sections]]
//! name = "letters"
//! required = true            # can never be turned off
//! rows = [["a", { tap = ".", hold = ".." }, "<layer:nav>"]]
//!
//! [[layers]]
//! name = "base"
//! sections = ["numbers", "letters"]
//!
//! [[layers]]
//! name = "nav"               # a layer can also list its rows directly
//! rows = [["<left>", "<right>", "<layer:nav>"]]
//! ```
//!
//! A [`Layout`] is the definition: reusable sections, and layers made of them.
//! An [`Arrangement`] (from a profile) switches sections on and off and reorders
//! them; [`Layout::keymap`] applies one and gives the [`Keymap`] the keyboard
//! runs. Sections show in the order they are declared unless an arrangement
//! reorders them, and the order is the same on every layer.
//!
//! The first layer is the one shown when the keyboard starts. See
//! [`crate::action`] for the key notation and `layouts/dev.toml` for every option.

mod arrangement;
mod error;
mod keymap;
mod resolve;
mod schema;

pub use arrangement::Arrangement;
pub use error::{Block, KeyProblem, LayoutError};
pub use keymap::{KeyId, Keymap, Layer};

use crate::action::Action;

/// The developer layout that ships with Super Keyboard.
pub const DEV_LAYOUT_TOML: &str = include_str!("../../layouts/dev.toml");

/// A layout definition: sections, and the layers made of them.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub name: String,
    pub description: String,
    /// In their default order. Layers that list their rows directly get a
    /// required section named after the layer, after the declared ones.
    pub sections: Vec<Section>,
    /// Never empty; the first layer is the base layer.
    pub layers: Vec<LayerDef>,
}

/// A named block of rows that layers are built from, and that profiles switch
/// on and off and reorder.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: String,
    /// What settings screens call it.
    pub title: String,
    pub rows: Vec<Vec<Key>>,
    /// Whether it shows when no profile says otherwise.
    pub enabled: bool,
    /// Required sections are always on.
    pub required: bool,
}

/// A layer as defined: which sections it shows.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerDef {
    pub name: String,
    /// Indices into [`Layout::sections`], in declaration order.
    pub sections: Vec<usize>,
}

/// One key.
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pub label: String,
    /// What a tap does.
    pub tap: Action,
    /// What a long press does instead, if anything.
    pub hold: Option<Action>,
    /// What dragging across the key does, if anything.
    pub swipe: Option<Swipe>,
    /// Whether holding the key repeats its tap action.
    pub repeat: bool,
    /// Relative width; a normal key is `1.0`.
    pub width: f32,
}

/// What dragging sideways across a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Swipe {
    /// Move the cursor one character per step, like a trackpad.
    Cursor,
}

impl Swipe {
    /// Looks up a swipe by the name used in layout files.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "cursor" => Some(Self::Cursor),
            _ => None,
        }
    }
}

impl Layout {
    /// The developer layout that ships with Super Keyboard.
    pub fn builtin() -> Self {
        Self::from_toml(DEV_LAYOUT_TOML).expect("the built-in layout is valid")
    }

    /// Parses and validates a layout file.
    pub fn from_toml(source: &str) -> Result<Self, LayoutError> {
        let file: schema::LayoutFile = toml::from_str(source)?;
        resolve::resolve(file)
    }

    /// Index of the layer with this name.
    pub fn layer_index(&self, name: &str) -> Option<usize> {
        self.layers.iter().position(|layer| layer.name == name)
    }

    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|section| section.name == name)
    }

    /// Section names in their default order.
    pub fn section_names(&self) -> Vec<&str> {
        self.sections.iter().map(|s| s.name.as_str()).collect()
    }

    /// The layers that show the section at `index` of [`Layout::sections`].
    pub fn layers_with(&self, index: usize) -> impl Iterator<Item = &str> {
        self.layers
            .iter()
            .filter(move |layer| layer.sections.contains(&index))
            .map(|layer| layer.name.as_str())
    }

    /// Sections in the order `arrangement` shows them, with whether each is on.
    pub fn arranged_sections(&self, arrangement: &Arrangement) -> Vec<(&Section, bool)> {
        arrangement
            .arrange(&self.section_names())
            .into_iter()
            .map(|index| {
                let section = &self.sections[index];
                (section, section.is_on(arrangement))
            })
            .collect()
    }

    /// The keymap for `arrangement`: each layer's sections that are on, in the
    /// arranged order.
    pub fn keymap(&self, arrangement: &Arrangement) -> Keymap {
        let order = arrangement.arrange(&self.section_names());
        let layers = self
            .layers
            .iter()
            .map(|layer| Layer {
                name: layer.name.clone(),
                rows: order
                    .iter()
                    .filter(|index| layer.sections.contains(index))
                    .map(|&index| &self.sections[index])
                    .filter(|section| section.is_on(arrangement))
                    .flat_map(|section| section.rows.iter().cloned())
                    .collect(),
            })
            .collect();
        Keymap {
            name: self.name.clone(),
            layers,
        }
    }

    /// Names in `arrangement` that are not sections of this layout.
    pub fn unknown_sections<'a>(&self, arrangement: &'a Arrangement) -> Vec<&'a str> {
        arrangement.unknown_names(&self.section_names())
    }
}

impl Section {
    /// Whether the section shows under `arrangement`. Required ones always do.
    pub fn is_on(&self, arrangement: &Arrangement) -> bool {
        self.required || arrangement.is_enabled(&self.name, self.enabled)
    }
}

impl Key {
    /// A key with default options: its own label, no hold or swipe, normal width,
    /// and repeating only if the action repeats by default.
    pub fn new(tap: Action) -> Self {
        Self {
            label: tap.default_label(),
            repeat: tap.repeats_by_default(),
            tap,
            hold: None,
            swipe: None,
            width: 1.0,
        }
    }

    /// The hint drawn on keys that have a hold action.
    pub fn hold_label(&self) -> Option<String> {
        self.hold.as_ref().map(Action::default_label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Modifier, NamedKey};

    fn layout(source: &str) -> Layout {
        Layout::from_toml(source).unwrap_or_else(|err| panic!("{err}\n{source}"))
    }

    /// The labels of each row of `layer`, joined.
    fn rows(keymap: &Keymap, layer: &str) -> Vec<String> {
        let layer = &keymap.layers[keymap.layer_index(layer).unwrap()];
        layer
            .rows
            .iter()
            .map(|row| row.iter().map(|k| k.label.as_str()).collect())
            .collect()
    }

    const SECTIONS: &str = r#"
        name = "s"
        [[sections]]
        name = "numbers"
        title = "Number row"
        enabled = false
        rows = [["1", "2"]]
        [[sections]]
        name = "letters"
        rows = [["a", "b"], ["c", "d"]]
        [[sections]]
        name = "bottom"
        required = true
        rows = [["<layer:sym>", "<space>"]]
        [[sections]]
        name = "symbols"
        rows = [["!", "?"]]
        [[layers]]
        name = "base"
        sections = ["numbers", "letters", "bottom"]
        [[layers]]
        name = "sym"
        sections = ["bottom", "symbols"]
    "#;

    #[test]
    fn builtin_layout_is_valid() {
        let layout = Layout::builtin();
        assert_eq!(layout.layers[0].name, "base");
        assert!(layout.layer_index("sym").is_some());
        assert!(layout.layer_index("nav").is_some());
        let keymap = Keymap::from(&layout);
        let space = keymap
            .keys()
            .find(|(_, key)| key.tap == Action::Key(NamedKey::Space))
            .expect("a space bar");
        assert_eq!(space.1.swipe, Some(Swipe::Cursor));
        assert!(layout.section("bottom").unwrap().required);
        let bottom = layout.section_names().iter().position(|n| *n == "bottom");
        let layers: Vec<_> = layout.layers_with(bottom.unwrap()).collect();
        assert_eq!(layers, ["base", "sym", "nav"]);
    }

    #[test]
    fn the_vietnamese_section_is_off_until_a_profile_turns_it_on() {
        let layout = Layout::builtin();
        let section = layout.section("vietnamese").expect("a vietnamese section");
        assert!(!section.enabled && !section.required);
        let index = layout
            .section_names()
            .iter()
            .position(|n| *n == "vietnamese");
        let layers: Vec<_> = layout.layers_with(index.unwrap()).collect();
        assert_eq!(layers, ["base"]);

        let off = Keymap::from(&layout);
        assert!(!rows(&off, "base").contains(&"ăâđêôơư".to_owned()));

        let mut arrangement = Arrangement::default();
        arrangement.set_enabled("vietnamese", true, false);
        let on = layout.keymap(&arrangement);
        let base = rows(&on, "base");
        let row = base.iter().position(|row| row == "ăâđêôơư");
        // One tap away: right above the letters.
        assert!(
            base[row.expect("the row") + 1].starts_with("qwerty"),
            "{base:?}"
        );
        assert_eq!(rows(&on, "sym"), rows(&off, "sym"));
        assert_eq!(rows(&on, "nav"), rows(&off, "nav"));
    }

    #[test]
    fn the_vietnamese_section_has_the_tones_on_long_press() {
        let layout = Layout::builtin();
        let keys = &layout.section("vietnamese").unwrap().rows[0];
        let holds: Vec<Option<String>> = keys
            .iter()
            .map(|key| key.hold.as_ref().map(Action::default_label))
            .collect();
        // Sắc, huyền, hỏi, ngã, nặng, in VNI's order, as combining marks
        // drawn on a dotted circle.
        let tone = |mark: char| Some(format!("\u{25cc}{mark}"));
        assert_eq!(
            holds,
            [
                tone('\u{301}'),
                tone('\u{300}'),
                tone('\u{309}'),
                tone('\u{303}'),
                tone('\u{323}'),
                None,
                None,
            ]
        );
        for key in keys {
            assert!(key.tap.single_char().is_some(), "{key:?}");
            assert_eq!(key.swipe, None);
        }
    }

    #[test]
    fn sections_build_layers_in_declaration_order() {
        let layout = layout(SECTIONS);
        let keymap = Keymap::from(&layout);
        assert_eq!(rows(&keymap, "base"), ["ab", "cd", "symspace"]);
        assert_eq!(rows(&keymap, "sym"), ["symspace", "!?"]);
        assert_eq!(layout.section("numbers").unwrap().title, "Number row");
        assert_eq!(layout.section("letters").unwrap().title, "letters");
    }

    #[test]
    fn arrangements_switch_and_reorder_sections_on_every_layer() {
        let layout = layout(SECTIONS);
        let mut arrangement = Arrangement::default();
        arrangement.set_enabled("numbers", true, false);
        arrangement.set_enabled("letters", false, true);
        arrangement.set_enabled("bottom", false, true);
        arrangement.order = vec!["symbols".into(), "bottom".into()];
        let keymap = layout.keymap(&arrangement);
        // Required sections stay on; `symbols` now comes before `bottom`.
        assert_eq!(rows(&keymap, "base"), ["12", "symspace"]);
        assert_eq!(rows(&keymap, "sym"), ["!?", "symspace"]);

        let shown: Vec<_> = layout
            .arranged_sections(&arrangement)
            .into_iter()
            .map(|(s, on)| (s.name.as_str(), on))
            .collect();
        assert_eq!(
            shown,
            [
                ("numbers", true),
                ("letters", false),
                ("symbols", true),
                ("bottom", true)
            ]
        );
    }

    #[test]
    fn layers_with_rows_get_a_required_section_of_their_own() {
        let layout = layout(
            r#"
            name = "t"
            [[layers]]
            name = "a"
            rows = [["1", "2"], ["3"]]
            [[layers]]
            name = "b"
            rows = [["4"]]
            "#,
        );
        assert_eq!(layout.section_names(), ["a", "b"]);
        assert!(layout.sections.iter().all(|s| s.required));
        let keymap = Keymap::from(&layout);
        let id = KeyId {
            layer: 0,
            row: 1,
            col: 0,
        };
        assert_eq!(keymap.key(id).map(|k| k.label.as_str()), Some("3"));
        assert_eq!(
            keymap.key(KeyId {
                layer: 2,
                row: 0,
                col: 0
            }),
            None
        );
        let labels: Vec<_> = keymap.keys().map(|(_, k)| k.label.as_str()).collect();
        assert_eq!(labels, ["1", "2", "3", "4"]);
    }

    #[test]
    fn reports_unknown_sections_in_an_arrangement() {
        let layout = layout(SECTIONS);
        let mut arrangement = Arrangement::default();
        arrangement.set_enabled("nope", true, false);
        assert_eq!(layout.unknown_sections(&arrangement), ["nope"]);
    }

    #[test]
    fn parses_short_and_table_keys() {
        let keymap = Keymap::from(&layout(
            r#"
            name = "t"
            [[layers]]
            name = "base"
            rows = [["<ctrl>", { tap = "<space>", width = 3.0 }, { text = "<div>", label = "div" }]]
            "#,
        ));
        let row = &keymap.layers[0].rows[0];
        assert_eq!(row[0].tap, Action::Modifier(Modifier::Ctrl));
        assert_eq!(row[0].label, "ctrl");
        assert_eq!(row[1].tap, Action::Key(NamedKey::Space));
        assert_eq!(row[1].width, 3.0);
        assert_eq!(row[2].tap, Action::Text("<div>".to_owned()));
        assert_eq!(row[2].label, "div");
    }

    #[test]
    fn parses_hold_swipe_and_repeat() {
        let keymap = Keymap::from(&layout(
            r#"
            name = "t"
            [[layers]]
            name = "base"
            rows = [[
                { tap = ".", hold = ".." },
                { tap = "<space>", swipe = "cursor" },
                "<bksp>",
                { tap = "<bksp>", repeat = false },
                { tap = "x", repeat = true },
                { tap = "<left>", hold = "<home>" },
            ]]
            "#,
        ));
        let row = &keymap.layers[0].rows[0];
        assert_eq!(row[0].hold, Some(Action::Text("..".into())));
        assert_eq!(row[0].hold_label().as_deref(), Some(".."));
        assert_eq!(row[1].swipe, Some(Swipe::Cursor));
        assert!(row[2].repeat);
        assert!(!row[3].repeat);
        assert!(row[4].repeat);
        assert!(!row[5].repeat, "a hold action turns off default repeat");
    }
}
