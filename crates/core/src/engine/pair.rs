//! Auto-pairing: an opening bracket or quote types its closer too.
//!
//! [`AutoPair`] sits between the Vietnamese [`Composer`](crate::Composer) and the
//! sink, so a bracket or quote the input method marks with (VIQR `(`, Telex `[`)
//! is already a letter when it gets here. It only ever sends `Text` and the
//! Left, Right and Delete keys, so every sink gets auto-pairing for free.

use crate::{
    action::{Modifiers, NamedKey, is_combining_mark},
    output::{KeyCode, Output},
};

/// What each opener is paired with. A quote opens and closes with the same
/// character.
const PAIRS: [(char, char); 6] = [
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('"', '"'),
    ('\'', '\''),
    ('`', '`'),
];

/// A pair the cursor is inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Open {
    close: char,
    /// How many characters were typed between the opener and the cursor, at
    /// most. While it is zero, Backspace removes both halves of the pair.
    typed: usize,
}

/// Turns `(` into `()` with the cursor between them, and likewise for
/// `[`, `{`, `"`, `'` and `` ` ``.
///
/// - Typing the closer of the pair the cursor is in steps over it, so typing
///   `(x)` out of habit gives `(x)`, not `(x))`.
/// - Backspace right after an opener removes the closer too.
/// - A quote right after a letter or digit is typed alone: it is an
///   apostrophe (`don't`) or closes a quote typed some other way.
/// - Only a single typed character pairs. Longer text (`$(`, `->`) and chords
///   pass through unchanged.
///
/// The keyboard cannot see the text it types into, so it only remembers pairs
/// until the cursor moves: an arrow key, a chord or any other key forgets
/// them, and their closers are then ordinary characters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AutoPair {
    enabled: bool,
    /// The pairs the cursor is inside, innermost last.
    open: Vec<Open>,
    /// The last thing typed was a letter or digit.
    after_word: bool,
}

impl AutoPair {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Switches auto-pairing on or off, forgetting the pairs in progress.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.reset();
    }

    /// Forgets the pairs in progress, for when the text they were typed into
    /// changed some other way.
    pub fn reset(&mut self) {
        self.open.clear();
        self.after_word = false;
    }

    /// What to send for `output`.
    pub fn process(&mut self, output: Output) -> Vec<Output> {
        if !self.enabled {
            return vec![output];
        }
        match &output {
            Output::Text(text) => {
                let mut chars = text.chars();
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    return self.character(c);
                }
                self.typed(text.chars().count());
                // A combining mark belongs to the letter before it, as in a
                // word the composer writes decomposed.
                if let Some(last) = text.chars().rev().find(|c| !is_combining_mark(*c)) {
                    self.after_word = last.is_alphanumeric();
                }
            }
            Output::Key {
                key: KeyCode::Named(key),
                mods,
            } if mods.is_shift_only() => match key {
                NamedKey::Backspace => return self.backspace(output),
                // These type a character where the cursor is.
                NamedKey::Space | NamedKey::Enter | NamedKey::Tab => {
                    self.typed(1);
                    self.after_word = false;
                }
                _ => self.reset(),
            },
            // A chord, or a key that moves the cursor: the keyboard no longer
            // knows what is around it.
            Output::Key { .. } => self.reset(),
        }
        vec![output]
    }

    /// A single typed character: a closer to step over, an opener to pair, or
    /// just a character.
    fn character(&mut self, c: char) -> Vec<Output> {
        if let Some(inner) = self.open.pop_if(|open| open.close == c) {
            // The pair is now text inside the pair around it.
            self.typed(inner.typed + 2);
            self.after_word = false;
            return vec![Output::named(NamedKey::Right, Modifiers::NONE)];
        }
        let pair = PAIRS.iter().find(|(open, _)| *open == c);
        if let Some(&(open, close)) = pair
            && !(open == close && self.after_word)
        {
            self.open.push(Open { close, typed: 0 });
            self.after_word = false;
            return vec![
                Output::Text([open, close].iter().collect()),
                Output::named(NamedKey::Left, Modifiers::NONE),
            ];
        }
        self.typed(1);
        // A combining mark (a tone from the Vietnamese row) belongs to the
        // letter before it, so it leaves `after_word` as it was.
        if !is_combining_mark(c) {
            self.after_word = c.is_alphanumeric();
        }
        vec![Output::Text(c.into())]
    }

    fn backspace(&mut self, backspace: Output) -> Vec<Output> {
        self.after_word = false;
        match self.open.last_mut() {
            Some(open) if open.typed == 0 => {
                self.open.pop();
                vec![backspace, Output::named(NamedKey::Delete, Modifiers::NONE)]
            }
            Some(open) => {
                open.typed -= 1;
                vec![backspace]
            }
            None => vec![backspace],
        }
    }

    /// Counts `count` characters typed inside the innermost pair.
    fn typed(&mut self, count: usize) {
        if let Some(open) = self.open.last_mut() {
            open.typed += count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        action::Modifier,
        output::{OutputSink, PreviewSink},
        vietnamese::{Composer, VietnameseOptions},
    };

    fn text(s: &str) -> Output {
        Output::Text(s.into())
    }

    fn key(key: NamedKey) -> Output {
        Output::named(key, Modifiers::NONE)
    }

    /// One output per character of `keys`; a space is the Space key, `\n`
    /// Enter, `\u{8}` Backspace, `←` and `→` the arrows.
    fn outputs(keys: &str) -> Vec<Output> {
        keys.chars()
            .map(|c| match c {
                ' ' => key(NamedKey::Space),
                '\n' => key(NamedKey::Enter),
                '\u{8}' => key(NamedKey::Backspace),
                '←' => key(NamedKey::Left),
                '→' => key(NamedKey::Right),
                c => text(&c.to_string()),
            })
            .collect()
    }

    /// Types `keys` with auto-pairing on and returns the text, with `|` where
    /// the cursor is.
    fn typed(keys: &str) -> String {
        let mut pairs = AutoPair::new(true);
        let mut sink = PreviewSink::default();
        for output in outputs(keys) {
            for output in pairs.process(output) {
                sink.emit(output);
            }
        }
        let view = sink.buffer().view();
        format!("{}|{}", view.before, view.after)
    }

    #[test]
    fn off_passes_everything_through() {
        let mut pairs = AutoPair::default();
        assert!(!pairs.enabled());
        assert_eq!(pairs.process(text("(")), [text("(")]);
        assert_eq!(pairs.process(text(")")), [text(")")]);
    }

    #[test]
    fn an_opener_types_the_pair_and_steps_back() {
        let pairs = [
            ("(", "()"),
            ("[", "[]"),
            ("{", "{}"),
            ("\"", "\"\""),
            ("'", "''"),
            ("`", "``"),
        ];
        for (opener, pair) in pairs {
            let mut auto = AutoPair::new(true);
            assert_eq!(
                auto.process(text(opener)),
                [text(pair), key(NamedKey::Left)],
                "{opener}"
            );
        }
        assert_eq!(typed("f(x"), "f(x|)");
    }

    #[test]
    fn typing_the_closer_steps_over_it() {
        let mut pairs = AutoPair::new(true);
        pairs.process(text("("));
        pairs.process(text("x"));
        assert_eq!(pairs.process(text(")")), [key(NamedKey::Right)]);
        // Nothing is open any more: the next `)` is typed.
        assert_eq!(pairs.process(text(")")), [text(")")]);
        assert_eq!(typed("f(x)"), "f(x)|");
        assert_eq!(typed("f(x);"), "f(x);|");
        assert_eq!(typed("{[()]}"), "{[()]}|");
        assert_eq!(typed("v[i] = (a, b)"), "v[i] = (a, b)|");
        assert_eq!(typed("if x {\ny\n}"), "if x {\ny\n}|");
    }

    #[test]
    fn a_quote_opens_closes_or_is_an_apostrophe() {
        assert_eq!(typed("\"hi"), "\"hi|\"");
        assert_eq!(typed("\"hi\""), "\"hi\"|");
        assert_eq!(typed("'a' `b`"), "'a' `b`|");
        // After a letter or digit a quote is an apostrophe or a closing quote.
        assert_eq!(typed("don't"), "don't|");
        assert_eq!(typed("5'"), "5'|");
        assert_eq!(typed("(\"\")"), "(\"\")|");
        // A combining mark belongs to the letter before it: a tone typed from
        // the Vietnamese row, or a word the composer writes decomposed.
        assert_eq!(typed("ca\u{301}'"), "ca\u{301}'|");
        let mut pairs = AutoPair::new(true);
        let mut sink = PreviewSink::default();
        for output in [text("ca\u{301}"), text("'")] {
            for output in pairs.process(output) {
                sink.emit(output);
            }
        }
        assert_eq!(sink.text(), "ca\u{301}'");
        // A mark after an opener is not a word.
        assert_eq!(typed("(\u{301}'"), "(\u{301}'|')");
    }

    #[test]
    fn a_closer_with_nothing_open_is_typed() {
        assert_eq!(typed(")"), ")|");
        assert_eq!(typed("a]"), "a]|");
        let mut pairs = AutoPair::new(true);
        assert_eq!(pairs.process(text("}")), [text("}")]);
    }

    #[test]
    fn backspace_right_after_an_opener_takes_the_closer_too() {
        let mut pairs = AutoPair::new(true);
        pairs.process(text("("));
        assert_eq!(
            pairs.process(key(NamedKey::Backspace)),
            [key(NamedKey::Backspace), key(NamedKey::Delete)]
        );
        assert_eq!(typed("a(\u{8}"), "a|");
        assert_eq!(typed("([\u{8}\u{8}"), "|");
        // With something typed inside, Backspace is only Backspace, until
        // the pair is empty again.
        assert_eq!(typed("(xy\u{8}"), "(x|)");
        assert_eq!(typed("(x\u{8}"), "(|)");
        assert_eq!(typed("(x\u{8}\u{8}"), "|");
        assert_eq!(typed("(x\u{8}y)"), "(y)|");
        // A pair that was closed is text like any other.
        assert_eq!(typed("([])\u{8}"), "([]|");
        assert_eq!(typed("([]\u{8}\u{8}\u{8}"), "|");
    }

    #[test]
    fn moving_the_cursor_forgets_open_pairs() {
        assert_eq!(typed("(←)"), ")|()");
        assert_eq!(typed("(→)"), "())|");
        let mut pairs = AutoPair::new(true);
        pairs.process(text("("));
        pairs.process(key(NamedKey::Escape));
        assert_eq!(pairs.process(text(")")), [text(")")]);

        pairs.process(text("("));
        let ctrl_backspace =
            Output::named(NamedKey::Backspace, Modifiers::NONE.with(Modifier::Ctrl));
        assert_eq!(pairs.process(ctrl_backspace.clone()), [ctrl_backspace]);
        assert_eq!(pairs.process(text(")")), [text(")")]);
    }

    #[test]
    fn longer_text_and_chords_are_left_alone() {
        let mut pairs = AutoPair::new(true);
        for output in [text("$("), text("${"), text("->"), text("()")] {
            assert_eq!(pairs.process(output.clone()), [output]);
        }
        let chord = Output::Key {
            key: KeyCode::Char('('),
            mods: Modifiers::NONE.with(Modifier::Ctrl),
        };
        assert_eq!(pairs.process(chord.clone()), [chord]);
        // `$(` left nothing open, so `)` is typed.
        assert_eq!(typed("a"), "a|");
        let mut pairs = AutoPair::new(true);
        pairs.process(text("$("));
        assert_eq!(pairs.process(text(")")), [text(")")]);
    }

    #[test]
    fn switching_off_or_resetting_forgets_open_pairs() {
        let mut pairs = AutoPair::new(true);
        pairs.process(text("("));
        pairs.reset();
        assert_eq!(pairs.process(text(")")), [text(")")]);

        pairs.process(text("("));
        pairs.set_enabled(false);
        pairs.set_enabled(true);
        assert_eq!(pairs.process(text(")")), [text(")")]);
    }

    #[test]
    fn vietnamese_composes_inside_a_pair() {
        let mut pairs = AutoPair::new(true);
        let mut composer = Composer::new(VietnameseOptions {
            enabled: true,
            ..VietnameseOptions::default()
        });
        let mut sink = PreviewSink::default();
        for output in outputs("(\"Vieejt\")") {
            for output in composer.process(output) {
                for output in pairs.process(output) {
                    sink.emit(output);
                }
            }
        }
        assert_eq!(sink.text(), "(\"Việt\")");
        assert_eq!(sink.buffer().view().after, "");
    }

    /// Through the composer first, as the app wires it: a VIQR mark key is a
    /// mark after a letter and an opener anywhere else.
    #[test]
    fn viqr_marks_are_marks_and_openers_still_pair() {
        let mut pairs = AutoPair::new(true);
        let mut composer = Composer::new(VietnameseOptions {
            enabled: true,
            method: crate::InputMethod::Viqr,
            ..VietnameseOptions::default()
        });
        let mut sink = PreviewSink::default();
        for output in outputs("(a` (b") {
            for output in composer.process(output) {
                for output in pairs.process(output) {
                    sink.emit(output);
                }
            }
        }
        assert_eq!(sink.text(), "(à (b))");
        assert_eq!(sink.buffer().view().after, "))");
        // Backspace takes the b, then the empty inner pair as a whole, then
        // the space.
        for _ in 0..3 {
            for output in composer.process(key(NamedKey::Backspace)) {
                for output in pairs.process(output) {
                    sink.emit(output);
                }
            }
        }
        assert_eq!(sink.text(), "(à)");
        assert_eq!(sink.buffer().view().after, ")");
    }
}
