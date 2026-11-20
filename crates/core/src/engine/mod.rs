//! The keyboard state machine: current layer and modifier latches.
//!
//! [`Keyboard`] knows *what* a key does. *When* it fires (tap, long-press,
//! repeat, swipe) is decided by [`crate::gesture`], which hands it [`Trigger`]s.
//! [`AutoPair`] then turns an opening bracket or quote into the pair.

mod chord;
mod latch;
mod pair;

pub use latch::Latch;
pub use pair::AutoPair;

use std::borrow::Cow;

use crate::{
    action::{Action, Modifier, Modifiers},
    gesture::Trigger,
    layout::{Key, KeyId, Keymap, Layer, Swipe},
    output::Output,
};
use latch::Latches;

/// A keyboard: a keymap plus what is currently active on it.
#[derive(Debug, Clone)]
pub struct Keyboard {
    keymap: Keymap,
    layer: usize,
    latches: Latches,
    /// The last output a key produced, so that holding a key repeats exactly what
    /// its first press sent (`Shift+Left` keeps selecting after a one-shot Shift
    /// has been released).
    last: Option<(KeyId, Output)>,
}

impl Keyboard {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            layer: 0,
            latches: Latches::default(),
            last: None,
        }
    }

    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    /// Switches to another keymap (another layout, or the same one rearranged),
    /// keeping modifier latches and, if it has a layer with the same name, the
    /// current layer.
    pub fn set_keymap(&mut self, keymap: Keymap) {
        self.layer = keymap.layer_index(&self.layer().name).unwrap_or(0);
        self.keymap = keymap;
        self.last = None;
    }

    /// The layer currently showing.
    pub fn layer(&self) -> &Layer {
        &self.keymap.layers[self.layer]
    }

    /// Goes back to the base layer, as tapping the current layer's key would.
    pub fn show_base_layer(&mut self) {
        self.layer = 0;
        self.last = None;
    }

    /// Index of the layer currently showing.
    pub fn layer_index(&self) -> usize {
        self.layer
    }

    /// The id of the key at `row`, `col` of the layer currently showing.
    pub fn key_id(&self, row: usize, col: usize) -> KeyId {
        KeyId {
            layer: self.layer,
            row,
            col,
        }
    }

    pub fn key(&self, id: KeyId) -> Option<&Key> {
        self.keymap.key(id)
    }

    pub fn latch(&self, modifier: Modifier) -> Latch {
        self.latches.get(modifier)
    }

    /// Modifiers that will apply to the next key.
    pub fn active_modifiers(&self) -> Modifiers {
        self.latches.active()
    }

    /// The label to draw for a key, taking Shift into account.
    pub fn display_label<'a>(&self, key: &'a Key) -> Cow<'a, str> {
        shifted_label(&key.label, &key.tap, self.is_shifted())
    }

    /// The small hint drawn on keys with a hold action, taking Shift into account.
    pub fn hold_hint(&self, key: &Key) -> Option<String> {
        let hold = key.hold.as_ref()?;
        let label = hold.default_label();
        Some(shifted_label(&label, hold, self.is_shifted()).into_owned())
    }

    /// Taps the key `id`.
    ///
    /// Returns what to send, or `None` when the tap only changed keyboard state
    /// (a modifier or layer key) or there is no such key.
    pub fn tap(&mut self, id: KeyId) -> Option<Output> {
        let action = self.key(id)?.tap.clone();
        self.fire(id, &action)
    }

    /// Long-presses the key `id`: runs its hold action, or its tap action if it
    /// has none.
    pub fn hold(&mut self, id: KeyId) -> Option<Output> {
        let key = self.key(id)?;
        let action = key.hold.as_ref().unwrap_or(&key.tap).clone();
        self.fire(id, &action)
    }

    /// Handles a trigger from [`crate::gesture::Gestures`].
    pub fn trigger(&mut self, trigger: Trigger) -> Option<Output> {
        match trigger {
            Trigger::Tap(id) => self.tap(id),
            Trigger::Hold(id) => self.hold(id),
            Trigger::Repeat(id) => match &self.last {
                Some((last, output)) if *last == id => Some(output.clone()),
                _ => self.tap(id),
            },
            Trigger::Swipe { key, direction } => match self.key(key)?.swipe? {
                // Modifiers apply to every step of a swipe, so Shift selects
                // as far as the finger goes; one-shots release at the end.
                Swipe::Cursor => Some(Output::named(direction.arrow(), self.latches.active())),
            },
            Trigger::SwipeEnd(_) => {
                self.latches.release_one_shots();
                None
            }
        }
    }

    /// Applies an action as if a key with it had been tapped.
    pub fn apply(&mut self, action: &Action) -> Option<Output> {
        // Whatever happens next, a held key must not repeat output that was
        // produced under different modifiers or on a different layer.
        self.last = None;
        match action {
            Action::Modifier(modifier) => {
                self.latches.cycle(*modifier);
                None
            }
            Action::Layer(name) => {
                let target = self.keymap.layer_index(name)?;
                self.layer = if self.layer == target { 0 } else { target };
                None
            }
            Action::Key(key) => Some(Output::named(*key, self.latches.consume())),
            Action::Text(text) => Some(chord::text_output(text, self.latches.consume())),
        }
    }

    fn fire(&mut self, id: KeyId, action: &Action) -> Option<Output> {
        let output = self.apply(action);
        self.last = output.clone().map(|output| (id, output));
        output
    }

    fn is_shifted(&self) -> bool {
        self.latch(Modifier::Shift).is_on()
    }
}

/// Upper-cases a label that shows the single character its action types.
fn shifted_label<'a>(label: &'a str, action: &Action, shifted: bool) -> Cow<'a, str> {
    match action.single_char() {
        Some(c) if shifted && label.chars().eq([c]) => label.to_uppercase().into(),
        _ => label.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{action::NamedKey, gesture::Direction, layout::Layout, output::KeyCode};

    fn keyboard() -> Keyboard {
        Keyboard::new(Keymap::builtin())
    }

    fn text(s: &str) -> Action {
        Action::Text(s.to_owned())
    }

    /// The id of the first key on the current layer whose tap is `action`.
    fn find(kb: &Keyboard, action: &Action) -> KeyId {
        kb.keymap()
            .keys()
            .find(|(id, key)| id.layer == kb.layer_index() && key.tap == *action)
            .map(|(id, _)| id)
            .unwrap_or_else(|| panic!("no {action:?} key"))
    }

    #[test]
    fn one_shot_ctrl_makes_a_chord_then_releases() {
        let mut kb = keyboard();
        assert_eq!(kb.apply(&Action::Modifier(Modifier::Ctrl)), None);
        assert_eq!(kb.latch(Modifier::Ctrl), Latch::OneShot);
        assert_eq!(
            kb.apply(&text("c")),
            Some(Output::Key {
                key: KeyCode::Char('c'),
                mods: Modifiers::NONE.with(Modifier::Ctrl),
            })
        );
        assert_eq!(kb.latch(Modifier::Ctrl), Latch::Off);
        assert_eq!(kb.apply(&text("c")), Some(Output::Text("c".into())));
    }

    #[test]
    fn shift_capitalises_once_and_locks_on_double_tap() {
        let mut kb = keyboard();
        let shift = Action::Modifier(Modifier::Shift);
        kb.apply(&shift);
        assert_eq!(kb.apply(&text("a")), Some(Output::Text("A".into())));
        assert_eq!(kb.apply(&text("a")), Some(Output::Text("a".into())));

        kb.apply(&shift);
        kb.apply(&shift);
        assert_eq!(kb.latch(Modifier::Shift), Latch::Locked);
        assert_eq!(kb.apply(&text("a")), Some(Output::Text("A".into())));
        assert_eq!(kb.apply(&text("b")), Some(Output::Text("B".into())));
        kb.apply(&shift);
        assert_eq!(kb.latch(Modifier::Shift), Latch::Off);
    }

    #[test]
    fn modifiers_combine_and_apply_to_named_keys() {
        let mut kb = keyboard();
        kb.apply(&Action::Modifier(Modifier::Shift));
        assert_eq!(
            kb.apply(&Action::Key(NamedKey::Tab)),
            Some(Output::named(
                NamedKey::Tab,
                Modifiers::NONE.with(Modifier::Shift)
            ))
        );
        kb.apply(&Action::Modifier(Modifier::Ctrl));
        kb.apply(&Action::Modifier(Modifier::Shift));
        let out = kb.apply(&text("t")).unwrap();
        assert_eq!(out.to_string(), "Ctrl+Shift+T");
    }

    #[test]
    fn multi_character_text_is_inserted_as_written() {
        let mut kb = keyboard();
        kb.apply(&Action::Modifier(Modifier::Shift));
        assert_eq!(kb.apply(&text("->")), Some(Output::Text("->".into())));
        assert!(kb.active_modifiers().is_empty());
    }

    #[test]
    fn layer_keys_toggle_between_a_layer_and_base() {
        let mut kb = keyboard();
        let sym = Action::Layer("sym".into());
        assert_eq!(kb.apply(&sym), None);
        assert_eq!(kb.layer().name, "sym");
        kb.apply(&sym);
        assert_eq!(kb.layer().name, "base");
    }

    #[test]
    fn tap_uses_the_keys_layer_and_ignores_missing_keys() {
        let mut kb = keyboard();
        assert_eq!(kb.tap(kb.key_id(99, 0)), None);
        let q = find(&kb, &text("q"));
        kb.apply(&Action::Layer("sym".into()));
        // The id still names the base-layer `q`, even though `sym` is showing.
        assert_eq!(kb.tap(q), Some(Output::Text("q".into())));
    }

    #[test]
    fn hold_runs_the_hold_action_or_falls_back_to_tap() {
        let mut kb = keyboard();
        let dot = find(&kb, &text("."));
        assert_eq!(kb.hold(dot), Some(Output::Text("..".into())));
        let esc = find(&kb, &Action::Key(NamedKey::Escape));
        assert_eq!(
            kb.hold(esc),
            Some(Output::named(NamedKey::Escape, Modifiers::NONE))
        );
    }

    #[test]
    fn repeat_resends_the_first_output_of_the_held_key() {
        let mut kb = keyboard();
        let bksp = find(&kb, &Action::Key(NamedKey::Backspace));
        kb.apply(&Action::Modifier(Modifier::Ctrl));
        let first = kb.trigger(Trigger::Tap(bksp));
        let ctrl_bksp = Output::named(NamedKey::Backspace, Modifiers::NONE.with(Modifier::Ctrl));
        assert_eq!(first.as_ref(), Some(&ctrl_bksp));
        assert_eq!(kb.latch(Modifier::Ctrl), Latch::Off);
        assert_eq!(kb.trigger(Trigger::Repeat(bksp)), Some(ctrl_bksp));
    }

    #[test]
    fn swipes_move_the_cursor_and_keep_shift_until_the_end() {
        let mut kb = keyboard();
        let space = find(&kb, &Action::Key(NamedKey::Space));
        kb.apply(&Action::Modifier(Modifier::Shift));
        let shift = Modifiers::NONE.with(Modifier::Shift);
        let step = |direction| Trigger::Swipe {
            key: space,
            direction,
        };
        for _ in 0..2 {
            assert_eq!(
                kb.trigger(step(Direction::Left)),
                Some(Output::named(NamedKey::Left, shift))
            );
        }
        assert_eq!(kb.trigger(Trigger::SwipeEnd(space)), None);
        assert_eq!(kb.latch(Modifier::Shift), Latch::Off);
        assert_eq!(
            kb.trigger(step(Direction::Right)),
            Some(Output::named(NamedKey::Right, Modifiers::NONE))
        );

        let q = find(&kb, &text("q"));
        assert_eq!(
            kb.trigger(Trigger::Swipe {
                key: q,
                direction: Direction::Left
            }),
            None
        );
    }

    #[test]
    fn labels_follow_shift() {
        let mut kb = keyboard();
        let key = Key::new(text("a"));
        let named = Key {
            label: "alpha".into(),
            ..Key::new(text("a"))
        };
        let q = kb.key(find(&kb, &text("q"))).unwrap().clone();
        assert_eq!(kb.display_label(&key), "a");
        kb.apply(&Action::Modifier(Modifier::Shift));
        assert_eq!(kb.display_label(&key), "A");
        assert_eq!(kb.display_label(&named), "alpha");
        assert_eq!(kb.hold_hint(&q).as_deref(), Some("1"));
    }

    #[test]
    fn vietnamese_letters_follow_shift_and_tones_join_the_letter_before() {
        use crate::{Arrangement, OutputSink, PreviewSink};
        let mut arrangement = Arrangement::default();
        arrangement.set_enabled("vietnamese", true, false);
        let mut kb = Keyboard::new(Layout::builtin().keymap(&arrangement));
        let mut sink = PreviewSink::default();
        let a_breve = find(&kb, &text("ă"));
        let e_circumflex = find(&kb, &text("ê"));
        let o_circumflex = find(&kb, &text("ô"));

        // "Việt": the nặng tone is a long-press of the fifth key.
        kb.apply(&Action::Modifier(Modifier::Shift));
        let outputs = [
            kb.apply(&text("v")),
            kb.apply(&text("i")),
            kb.tap(e_circumflex),
            kb.hold(o_circumflex),
            kb.apply(&text("t")),
        ];
        for output in outputs.into_iter().flatten() {
            sink.emit(output);
        }
        assert_eq!(sink.text(), "Viê\u{323}t");

        kb.apply(&Action::Modifier(Modifier::Shift));
        assert_eq!(kb.display_label(kb.key(a_breve).unwrap()), "Ă");
        assert_eq!(kb.tap(a_breve), Some(Output::Text("Ă".into())));
        let key = kb.key(a_breve).unwrap().clone();
        assert_eq!(kb.hold_hint(&key).as_deref(), Some("\u{25cc}\u{301}"));
        // Shift does not change a tone mark.
        kb.apply(&Action::Modifier(Modifier::Shift));
        assert_eq!(kb.hold(a_breve), Some(Output::Text("\u{301}".into())));
    }

    #[test]
    fn set_keymap_keeps_the_layer_by_name() {
        let mut kb = keyboard();
        kb.apply(&Action::Layer("nav".into()));
        kb.apply(&Action::Modifier(Modifier::Alt));
        kb.set_keymap(Keymap::builtin());
        assert_eq!(kb.layer().name, "nav");
        assert_eq!(kb.latch(Modifier::Alt), Latch::OneShot);

        let other =
            Layout::from_toml("name = \"x\"\n[[layers]]\nname = \"a\"\nrows = [[\"x\"]]").unwrap();
        kb.set_keymap(Keymap::from(&other));
        assert_eq!(kb.layer().name, "a");
    }

    #[test]
    fn show_base_layer_returns_from_any_layer() {
        let mut kb = keyboard();
        kb.apply(&Action::Layer("sym".into()));
        kb.show_base_layer();
        assert_eq!(kb.layer().name, "base");
    }
}
