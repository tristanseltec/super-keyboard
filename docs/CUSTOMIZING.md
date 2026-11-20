# Customizing Super Keyboard

Super Keyboard is configured with plain files: one `settings.toml`, and any number of
layouts and themes. The app's **settings** screen edits `settings.toml` for you;
everything else is a text editor away. On the desktop, the app reloads when you save a
file.

Three ideas make the keyboard modular:

- **Sections** are named blocks of keys in a layout: the number row, the bracket row,
  terminal keys, letters, the space-bar row. Layers are built from sections.
- **Profiles** pick a layout and arrange its sections: switch any section on or off and
  move it up or down. One order applies to every layer.
- **Presets** are profiles that ship with the app. Change one and you get your own copy;
  "reset to preset" brings the original back. The profile menu lists presets, your
  profiles and your layouts, in an order you choose, and can hide any of them.

## Where the files live

| Platform | Directory |
|---|---|
| Linux (Ubuntu) | `$XDG_CONFIG_HOME/super-keyboard`, usually `~/.config/super-keyboard` |
| Android | The app's private storage (`/data/user/0/<package>/files`); the settings screen is the way to change it |
| Anywhere | `$SUPER_KEYBOARD_CONFIG_DIR`, if set, wins |

```text
~/.config/super-keyboard/
├── settings.toml        profile, theme, key timing, keyboard size, style, profiles, menu
├── layouts/
│   ├── dev.toml         optional: replaces the built-in `dev` layout
│   └── vim.toml         another layout, chosen as "vim"
└── themes/
    ├── nord.json        optional: replaces the built-in `nord` theme
    └── midnight.json    another theme, chosen as "midnight"
```

Nothing has to exist. The directory and `settings.toml` are created the first time you
change a setting in the app, with every option written out and explained.

## When something is wrong

A broken file never stops the keyboard. The app shows what is wrong and where, under
the top bar:

```text
layouts/vim.toml       layer `base`, row 2, key 4: unknown key `<esc2>`
settings.toml          `keys.repeat-rate` must be between 5 and 60, not 100
settings.toml          profile `mine`: layout `dev` has no section `numbers`
themes/midnight.json   `colors.acent` is not a theme colour (known: bg, panel, key, ...)
```

While you edit, the last good version of a broken file stays in use, so saving a
half-finished layout does not reset the keyboard. Fix the file and save again.

## settings.toml

Every option is optional. Unknown names and out-of-range values are reported rather
than ignored, so typos do not go unnoticed. The app keeps your comments and formatting
when it saves.

```toml
# Profile to use: a preset, one of your [[profiles]], or a layout file's name.
profile = "coding"

# Theme: "system" follows the desktop or phone, or name one theme to always use.
theme = "system"
light-theme = "light"   # what "system" shows while the system is light
dark-theme = "nord"     # ... and while it is dark

[keys]
hold-delay = 300      # milliseconds before a long-press fires a key's hold action (150-1000)
repeat-delay = 400    # milliseconds before Backspace, Delete and the arrows repeat (150-1000)
repeat-rate = 30      # repeats per second (5-60)
swipe-step = 12       # pixels of drag per cursor step on the space bar (4-40)
height = "auto"       # "auto" fits keys to the screen, or a number of pixels (32-96)
auto-pair = false     # typing ( [ { " ' ` types the closer too (see Auto-pair below)

[keyboard]            # see Keyboard size below
width = "auto"        # "auto", or a percentage of the window width (40-100)
height = "auto"       # "auto", or a percentage of the window height (20-100)
align = "center"      # "center", "left" or "right"

[preview]
show-log = true       # show recent output beside the preview text

[vietnamese]          # see Vietnamese typing below
enabled = false       # the VI / EN button in the top bar switches this
method = "telex"      # "telex", "vni" or "viqr"
tone-style = "classic" # "classic" (hòa, thủy) or "modern" (hoà, thuỷ)
check-spelling = true # only change syllables that can be Vietnamese
telex-w = true        # Telex: a w with no vowel to mark types ư ("tw" is "tư")
telex-brackets = false # Telex: [ types ơ and ] types ư
unicode-form = "nfc"  # "nfc" (ệ is one character) or "nfd" (e and its marks)

[style]               # changes to every theme
radius = "8px"
key-font-family = "Fira Code, monospace"

[style.dark]          # only in dark themes
accent = "#ff9e64"

[style.light]         # only in light themes
accent = "#c2410c"

[menu]                # the profile menu (see Profiles below)
order = ["terminal", "coding"]
shown = { compact = false }

[[profiles]]          # your profiles, and your changes to presets
name = "mine"
title = "Mine"
layout = "dev"
sections = { number-row = true, brackets = false }
```

## Auto-pair

With `auto-pair = true` under `[keys]` (**settings → General → Auto-pair brackets and
quotes**), typing an opening bracket or quote types its closer too and leaves the cursor
between them: `(` types `()`, and likewise for `[`, `{`, `"`, `'` and `` ` ``.

- **Typing the closer steps over it**, so `(x)` typed out of habit is `(x)`, not `(x))`.
- **Backspace right after an opener removes both halves.**
- **A quote after a letter or digit is typed alone**: `don't` stays `don't`. A tone mark
  counts as part of the letter it is on, so `cá'` is not paired either.
- Only a single typed character pairs. Keys that insert longer text (`$(`, `${`) and
  chords such as `Ctrl+(` are not changed.
- The keyboard cannot see the text it types into, so it remembers a pair only until the
  cursor moves: after an arrow key or a shortcut, a closer is an ordinary character.

It is off by default: editors that pair brackets themselves (most IDEs, and shells with
a plugin for it) would double them. It works with Vietnamese typing: `("Vieejt")` is
`("Việt")`. A bracket or quote your input method uses as a mark stays a mark right
after a letter (VIQR `a(` is `ă`, `` a` `` is `à`; with Telex brackets on, `m[` is `mơ`)
and opens a pair anywhere else.

## Themes

A theme sets the colours, the fonts and the shape of the keys. Pick one on the
settings screen's **Themes** tab, where every theme is drawn in its own colours and
fonts:

- **Follow the system**: tap a dark theme for when the desktop or phone is dark, and a
  light theme for when it is light (`theme = "system"`, `dark-theme`, `light-theme`).
- **One theme**: tap the theme to use whatever the system says (`theme = "nord"`).

### Built-in themes

| Name | Appearance | Look | Fonts |
|---|---|---|---|
| `dark` (default) | dark | Cool greys, soft blue accent | JetBrains Mono |
| `tokyo-night` | dark | Deep indigo, neon blue | Inter, JetBrains Mono keys |
| `dracula` | dark | Purple and pink on charcoal | Fira Code |
| `nord` | dark | Arctic blue-greys and frost | Inter, medium-weight keys |
| `gruvbox-dark` | dark | Warm retro browns and yellow | Iosevka, semi-bold keys |
| `solarized-dark` | dark | Teal night, precise accents | Source Sans 3, Source Code Pro |
| `catppuccin-mocha` | dark | Pastel mauve on soft dark | Inter, Cascadia Code keys |
| `one-dark` | dark | The balanced editor classic | Inter, Source Code Pro |
| `rose-pine` | dark | Muted roses and pine on plum | Inter, Cascadia Code keys |
| `everforest-dark` | dark | Soft forest greens | Hack |
| `kanagawa` | dark | Ink and wave blues | Inter, Iosevka |
| `aubergine` | dark | Aubergine and orange | Ubuntu, Ubuntu Mono |
| `oled` | dark | True black for OLED phones, outlined keys | Roboto, Roboto Mono |
| `amber` | dark | Glowing amber CRT, square keys | VT323, Terminus |
| `phosphor` | dark | Green terminal glow, outlined square keys | Share Tech Mono, VT323 |
| `neon` | dark | Violet night, hot pink, glowing outlines | Exo 2, Share Tech Mono |
| `high-contrast-dark` | dark | White on black, outlined keys, yellow accent | Atkinson Hyperlegible, bold keys |
| `light` (default) | light | White keys on soft grey, deep blue accent | JetBrains Mono |
| `solarized-light` | light | Warm cream, precise accents | Source Sans 3, Source Code Pro |
| `gruvbox-light` | light | Retro parchment, earthy ink | Iosevka, semi-bold keys |
| `catppuccin-latte` | light | Milky pastels, mauve accent | Inter, Cascadia Code keys |
| `rose-pine-dawn` | light | Dawn pinks and pine on warm paper | Inter, Cascadia Code keys |
| `paper` | light | Warm sepia, serif interface | Literata or Georgia, IBM Plex Mono |
| `high-contrast-light` | light | Black on white, outlined keys | Atkinson Hyperlegible, bold keys |

Themes named after an editor colour scheme take their colours from that scheme's
published palette (all MIT licensed), with some shades adjusted so labels stay
readable. Every built-in theme keeps key labels at 7:1 contrast or better against
their keys, and secondary text, named-key labels and the accent at 4.5:1 or better; a
test checks this for every theme.

**Fonts are never downloaded.** A theme lists fonts in order of preference and ends
with a generic family such as `monospace` or `sans-serif`; the first one installed is
used. Install a font (on Ubuntu, `sudo apt install fonts-jetbrains-mono`, or copy it
to `~/.local/share/fonts`) to see a theme as designed.

### Your own themes

A theme is a JSON file in `themes/`. Its file name (without `.json`) is the name you
choose it by; a file named like a built-in theme replaces it. The easiest start is
**edit a copy** on the Themes tab, which saves the theme you use to `themes/` for you.
Or copy one from [`crates/config/themes/`](../crates/config/themes/).

```json
{
  "$schema": "https://raw.githubusercontent.com/tristanseltec/super-keyboard/main/crates/config/themes/theme.schema.json",
  "title": "Midnight",
  "description": "Shown as a tooltip in the theme list.",
  "appearance": "dark",
  "colors": {
    "bg": "#0b1020",
    "panel": "#080c18",
    "key": "#1b2340",
    "key-special": "#141a30",
    "accent": "#ffb454",
    "on-accent": "#0b1020"
  },
  "fonts": {
    "ui": { "family": ["Inter", "Ubuntu", "sans-serif"], "size": "13px" },
    "keys": { "family": ["Fira Code", "monospace"], "weight": 500 },
    "preview": { "family": "JetBrains Mono, monospace", "size": "15px" }
  },
  "sizes": { "radius": "10px", "gap": "4px" }
}
```

| Field | Required | Meaning |
|---|---|---|
| `title` | yes | What the theme list shows. |
| `appearance` | yes | `"dark"` or `"light"`. Decides which `[style.dark]` or `[style.light]` changes apply and where the theme is offered while following the system. |
| `description` | no | A sentence about the theme. |
| `colors` | no | Any of the colour variables below. |
| `fonts` | no | `ui` (top bar, settings, notices), `keys` (key labels) and `preview` (the typed text), each with a `family` (a list of fonts, or one string with commas), a `size` and a `weight` (`400`, `"bold"`). |
| `sizes` | no | `radius` and `gap`. |
| `$schema` | no | Ignored by the app; lets editors such as VS Code complete and check the file against [`theme.schema.json`](../crates/config/themes/theme.schema.json). |

Everything you leave out comes from the built-in theme with the same appearance
(`dark` or `light`), except key labels: `key-text` follows your `text` and
`key-special-text` your `muted`. So a theme can be three colours long. Unknown fields
and bad values are reported in the app with the field's name, like any other file.

### Style variables

`[style]` in `settings.toml` changes every theme; `[style.dark]` and `[style.light]`
change only dark or only light themes, and win over `[style]`. Themes set the same
variables (under `colors`, `fonts` and `sizes`).

| Variable | Kind | What it changes |
|---|---|---|
| `bg` | colour | app background |
| `panel` | colour | top bar and keyboard background |
| `key` | colour | key background |
| `key-special` | colour | background of Esc, modifier, layer and other named keys |
| `key-pressed` | colour | key background while pressed |
| `key-border` | colour | key outline (`transparent` for none) |
| `text` | colour | main text |
| `key-text` | colour | key labels |
| `key-special-text` | colour | labels of named keys |
| `muted` | colour | secondary text and hold hints |
| `accent` | colour | caret, one-shot modifiers and the active layer |
| `on-accent` | colour | label of a locked modifier, drawn on the accent colour |
| `selection` | colour | selected text in the preview |
| `warning` | colour | notices about layout and settings errors |
| `radius` | length | key corner radius |
| `gap` | length | space between keys |
| `font-family` | fonts | top bar, settings and notices |
| `font-size` | length | their text size |
| `font-weight` | weight | their text weight |
| `key-font-family` | fonts | key labels |
| `key-font-size` | length | key label size (`key-font` still works) |
| `key-font-weight` | weight | key label weight |
| `preview-font-family` | fonts | the typed text in the preview |
| `preview-font-size` | length | its size |
| `preview-font-weight` | weight | its weight |

Colours are CSS colours (`#7aa2f7`, `rgb(…)`, `hsl(…)`, `oklch(…)`, `color-mix(…)`,
colour names); lengths are `6px`, `0.5rem`, `calc(…)`, `clamp(…)`; weights are
`normal`, `bold` or a number from 1 to 1000; fonts are names separated by commas,
without quotes (`"Fira Code, monospace"`).

Key height is `keys.height`, not a style variable, because the settings screen controls it.

**Styles and themes can never load anything.** A keyboard must not be able to send
what it shows anywhere, so values may only use the characters and functions above:
`url(…)`, `image-set(…)`, `@import`, `@font-face`, quotes, `;`, braces and backslashes
are rejected with an error that names the value. Font names may only use letters,
digits, spaces and `._+&-`, and the app quotes them itself.

Check contrast when you change colours: aim for 4.5:1 or better between every label
and what it is drawn on.

## Keyboard size

By default the keyboard spans the window (at most 1100 pixels on wide screens) and its
keys follow the screen height. On the **General** tab, under *Keyboard size*, or in
`[keyboard]`:

| Setting | Values | What it does |
|---|---|---|
| Height: *Fit to screen* | `keys.height = "auto"` | Keys follow the screen height, at least 40 pixels. |
| Height: *Fixed key height* | `keys.height = 52` | Every row is that many pixels tall (32-96). |
| Height: *Share of window* | `keyboard.height = 45` | The keyboard fills that percentage of the window height (20-100) and its keys stretch to fill it, labels growing with them. Wins over `keys.height`. |
| Width | `keyboard.width = 80` | The keyboard spans that percentage of the window width (40-100). `"auto"` is the full width. |
| Position | `keyboard.align = "left"` | Where a narrower keyboard sits: `"center"`, `"left"` or `"right"`, for example to type with one hand on a big phone. |

`keyboard.height = 100` gives a **full keyboard**: it fills the window below the top
bar, hides the preview, and grows and shrinks as you resize the window. While the
settings panel is open it keeps up to half the window (420 pixels) for itself, so you
can always change the size back.

## Profiles and presets

A profile is a layout plus an arrangement of its sections. Choose one from the menu in
the top bar; build and arrange them on the settings screen:

- **Sections tab**: tick a section to show it, untick to hide it, and move it with ▲ ▼.
  Sections marked *always on* (letters, the space-bar row) can move but not be hidden, so
  you can always type and switch layers. The layer names next to each section show where
  it appears.
- **Profiles tab**: rename the active profile, copy it into a new one, choose another,
  delete your own, or reset a changed preset. Move entries with ▲ ▼ to reorder the menu,
  and untick one to hide it from the menu.

Tapping the layer name in the top bar always goes back to the base layer.

### Presets

| Preset | What it shows |
|---|---|
| `coding` (default) | Esc/Tab and brackets row, letters, space-bar row; symbols and navigation layers |
| `coding-numbers` | The same with a number row on top |
| `terminal` | A row with Esc, Tab, `~ \| / -` and arrows instead of the bracket row |
| `compact` | No bracket row, operator row or F-keys: more room on small phones |

Every layout file in `layouts/` that no profile uses also appears in the menu, under its
file name, so a new layout is always one tap away.

### Profiles in settings.toml

```toml
profile = "mine"

[[profiles]]
name = "mine"                       # letters, digits, - and _
title = "Mine"                      # what the menu shows (default: the name)
layout = "dev"                      # a layout's file name, without .toml
order = ["letters", "brackets"]     # these sections trade places, in this order
sections = { number-row = true, brackets = false }   # switch sections on or off
```

A profile named like a preset (`name = "coding"`) replaces that preset; delete the
`[[profiles]]` entry, or use "reset to preset", to get it back.

`order` lists sections in the order you want them. Listed sections take the places those
sections have in the layout, in your order; sections you leave out stay where they are.
The settings screen always writes the whole order. `sections` only needs the sections you
change: anything not listed keeps the layout's default, so a layout can gain sections
without breaking your profiles.

### The profile menu

```toml
[menu]
order = ["terminal", "mine", "coding"]   # same rule as a profile's `order`
shown = { compact = false, coding-numbers = false }   # hidden from the menu
```

The menu starts with the presets, then your profiles, then layouts that have no profile.
A hidden profile still works if `profile` names it; the menu shows it while it is in use.

## Vietnamese typing

Super Keyboard types Vietnamese with **Telex**, **VNI** or **VIQR**, the standard ways
of adding marks with an ordinary keyboard. Tap **EN** in the top bar to switch to **VI**,
or open **settings → Tiếng Việt** for the method, tone placement, spelling check,
Unicode form and a box to try keys in. The tab's cheat sheets are typed by the engine
itself, so they always match what the keys do.

| | Telex | VNI | VIQR | Result |
|---|---|---|---|---|
| Tones | `s` `f` `r` `x` `j` | `1` `2` `3` `4` `5` | `'` `` ` `` `?` `~` `.` | sắc á, huyền à, hỏi ả, ngã ã, nặng ạ |
| Remove the tone | `z` | `0` | `0` | a |
| Circumflex | `aa` `ee` `oo` | `a6` `e6` `o6` | `a^` `e^` `o^` | â ê ô |
| Breve | `aw` | `a8` | `a(` | ă |
| Horn | `ow` `uw` (or a lone `w`) | `o7` `u7` | `o+` `u+` | ơ ư |
| Stroke | `dd` | `d9` | `dd` | đ |
| Example | `Tieesng Vieejt` | `Tie61ng Vie65t` | `Tie^'ng Vie^.t` | Tiếng Việt |

- **Marks go anywhere in the syllable.** `tieengs`, `tiengse` and `tiengs` + `e` all type
  `tiếng`; `nguoiwf` and `nguwowif` both type `người`.
- **The tone moves with the syllable.** `hoaf` is `hòa`, and adding `n` makes `hoàn`.
  Backspace removes the last letter and puts the tone back where it belongs.
- **Twice takes it off.** Typing a mark's key again removes the mark and types the key:
  `ass` is `as`, `ww` is `w`, `a11` is `a1`, `a''` is `a'`. The rest of that word types
  as written.
- **Tone placement** (`tone-style`) only changes `oa`, `oe` and `uy` at the end of a
  syllable: classic `hòa khỏe thủy`, modern `hoà khoẻ thuỷ`.
- **Spelling check** (`check-spelling`) only changes syllables that can be Vietnamese,
  so `for`, `class`, `print` and most code type as written. Words such as `test`
  (`tét`) still look Vietnamese: switch to **EN** for English.
- A syllable ends at anything that is neither a letter nor a key the method marks with
  (VNI digits, VIQR punctuation, Telex brackets when they are on) and has something to
  mark: space, other punctuation, Enter, an arrow key or a `Ctrl` chord. Chords are never
  changed.
- **VIQR marks are punctuation**, so they only act right after a syllable they can mark:
  `ba.` is `bạ`, while `x.`, `f(x)`, `c++` and `3.5` type as written. At the end of a
  sentence:
  - a mark the word already has is the punctuation: `Vie^.t.` is `Việt.`, `Kho?e?` is
    `Khỏe?`;
  - any other mark changes the tone, and typed again at once is the punctuation instead:
    `kho^ng?` is `khổng`, `kho^ng??` is `không?`, `ba.n??` is `bạn?`;
  - a backslash after a syllable types the next mark as itself, as in RFC 1456:
    `kho^ng\?` is `không?`, `I\'m` is `I'm`, `ba\\` is `ba\`. Where there is no
    Vietnamese syllable before it, it stays a backslash: `C:\dev`, `\(x\)`,
    `example\.com`, `\\server`.
- **Words that turn out not to be Vietnamese go back to what you typed.** With the
  spelling check on, a mark made by VIQR punctuation or a Telex bracket is put back when a
  later letter makes the word impossible: `don't`, `let's`, `ma.com`, `e.g.` and `m[k]`
  type as written. A word that is still a Vietnamese syllable stays one (`I'm` is `Ím`;
  type `I\'m`). Only the last such mark is undone: `tie^'ng.txt` is `tiếng.txt`.
- **Telex options.** `telex-w = false` stops a `w` with no vowel to mark from typing
  `ư` (`tw` stays `tw`); `uw`, `ow` and `aw` still type `ư`, `ơ` and `ă`.
  `telex-brackets = true` makes `[` type `ơ` and `]` type `ư` (`th[` is `thơ`, `m]a` is
  `mưa`, `TH[` is `THƠ`); typing the bracket twice types the bracket. Brackets that
  start a syllable still make letters (`x[0]` is `xơ0ư`, `[1]` is `ơ1ư`), so leave this
  off if you type code. Typing the bracket again right after the `ơ` or `ư` it made types
  the bracket (`t[[i]` is `t[i]`). Both are on the Tiếng Việt tab while Telex is the
  method.
- **Unicode form** (`unicode-form`) is how letters with marks are written. `"nfc"`, the
  default, writes each one as a single precomposed character (*dựng sẵn*): `ệ` is
  U+1EC7. `"nfd"` writes the plain letter followed by its marks as combining characters
  (*tổ hợp*): `ệ` is `e`, U+0323, U+0302. Both look the same on screen. Almost every
  application expects NFC; choose NFD only for one that needs it. With NFD, Backspace on
  a letter of the syllable being typed still removes the whole letter.
- The app and the input method read the same `[vietnamese]` settings, so keep them the
  same version (the `.deb` installs both). An older one does not know `method = "viqr"`,
  `telex-w` or `telex-brackets` and ignores `settings.toml` until it is updated: it keeps
  the settings it had, or starts with the defaults. It never writes over the file.

### Vietnamese letters on the keyboard

To type Vietnamese without Telex or VNI, switch on the **Vietnamese letters** section
(`vietnamese`) on the **Sections tab**, or in a profile:

```toml
[[profiles]]
name = "viet"
title = "Tiếng Việt"
layout = "dev"
sections = { vietnamese = true }
```

It adds a row with `ă â đ ê ô ơ ư` right above the letters, off by default. Shift types
the capitals. Long-press the first five keys for the tones, in VNI's order:

| Long-press | `ă` | `â` | `đ` | `ê` | `ô` |
|---|---|---|---|---|---|
| Tone | sắc ◌́ | huyền ◌̀ | hỏi ◌̉ | ngã ◌̃ | nặng ◌̣ |

A tone goes on the letter before the cursor: `a` then long-press `ă` is `á`, and `V` `i`
`ê` long-press `ô` `t` is `Việt`. The tones are Unicode combining marks, so `á` typed
this way is stored as `a` followed by the mark (decomposed) rather than as one character.
It looks the same, but a few applications treat the two differently; Telex and VNI
always write single characters. The row is for the plain keys: with **VI** on, a letter
from it ends the syllable being typed.

### With the physical keyboard, in every app (Ubuntu)

The **Vietnamese (Super Keyboard)** input method types Telex, VNI or VIQR with your physical
keyboard in every application: terminals, browsers, editors, office apps. It is an IBus
engine (`apps/super-keyboard-ibus`) that uses the same rules and **the same
`[vietnamese]` settings** as the on-screen keyboard:

- Switch **VI / EN**, the method (Telex, VNI or VIQR), the tone style or the spelling check in the app,
  and the input method follows within a second; switch them from the **VI / EN menu
  in the desktop's top bar**, and the app follows. Both sides read and write
  `settings.toml`.
- The syllable being typed is shown underlined in the application and committed at a
  space, punctuation, Enter, an arrow or a shortcut (`Ctrl+S` saves the finished word).
  Backspace edits the syllable while it is underlined.
- **Super+Space** switches between it and your other input sources.

Install it with the `.deb` (it ships the engine), or from the source:

```sh
apps/super-keyboard-ibus/install.sh              # builds, installs (sudo), adds the input source
apps/super-keyboard-ibus/install.sh --uninstall
```

The installer puts the engine in `/usr/libexec/super-keyboard-ibus` and its component
file in `/usr/share/ibus/component/`, restarts IBus and adds the input source to GNOME.
After installing the `.deb`, run `ibus restart` (or log out and in), then add
**Vietnamese (Super Keyboard)** in **Settings → Keyboard → Input Sources**.

To try it without installing, run `target/release/super-keyboard-ibus --register` and
then `ibus engine super-keyboard`; `ibus engine xkb:us::eng` switches back.

The on-screen keys still type into the app's preview; typing into other apps with them
comes with the Android (v0.2) and Ubuntu desktop (v0.3) keyboards. The on-screen
keyboard rewrites text with Backspace and new text, the way Unikey does, so it will work
with every output backend.

## Layouts

A layout is a TOML file in `layouts/`. Its file name (without `.toml`) is the name you
pick it by, in the app's layout menu or in `settings.toml`. A file called `dev.toml`
replaces the built-in layout, so the easiest start is to copy
[`crates/core/layouts/dev.toml`](../crates/core/layouts/dev.toml) there and edit it.

```toml
name = "mine"
description = "Shown in the layout list."

[[sections]]
name = "numbers"              # how profiles refer to it: letters, digits, - and _
title = "Number row"          # what the settings screen shows (default: the name)
enabled = false               # off unless a profile turns it on (default: true)
rows = [["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]]

[[sections]]
name = "letters"
required = true               # always on; profiles can move it but not hide it
rows = [
    [{ tap = "q", hold = "1" }, "w", "e", "r", "t", "y"],
    ["<shift>", "a", "s", "d", "f", "<bksp>"],
]

[[sections]]
name = "bottom"
title = "Space bar and layer keys"
required = true
rows = [["<layer:nav>", { tap = "<space>", width = 3.0, swipe = "cursor" }, "<enter>"]]

[[layers]]                    # the first layer is the one shown at start
name = "base"
sections = ["numbers", "letters", "bottom"]

[[layers]]                    # a layer can list rows directly, as one fixed section
name = "nav"
rows = [["<home>", "<left>", "<up>", "<down>", "<right>", "<end>", "<layer:nav>"]]
```

Sections show in the order they are declared, and a layer lists the sections it uses
in that same order, so that one order can apply to every layer. A section can appear on
several layers: `bottom` above could be listed by `nav` too, and is then defined once.

Mark the rows you can never do without, such as the one with the layer keys, as
`required`: a profile can then never hide the way back to another layer.

### Key notation

| Written as | Meaning |
|---|---|
| `"a"`, `"->"`, `"::"` | Insert that text. One character responds to Shift and forms chords (`Ctrl+C`). |
| `"<esc>"`, `"<f5>"`, `"<up>"` | Press a named key. |
| `"<ctrl>"`, `"<alt>"`, `"<shift>"`, `"<super>"` | Modifier: tap once for the next key, twice to lock, three times to release. |
| `"<layer:sym>"` | Switch to a layer; tap again to go back to the first layer. |

Named keys: `esc`, `tab`, `enter` (`cr`), `bksp` (`bs`), `del`, `ins`, `space`, `up`,
`down`, `left`, `right`, `home`, `end`, `pgup`, `pgdn`, `f1`–`f24`.

Anything that is not `<name>` is text, so `"<="` and `"</"` need no escaping. TOML
literal strings (`'\'`, `'"'`) avoid escaping backslashes and quotes.

### Key options

Write a key as a table to set options:

| Option | Example | Meaning |
|---|---|---|
| `tap` | `{ tap = "<space>" }` | What a tap does, in key notation. |
| `text` | `{ text = "<div>" }` | Literal text, for text that looks like a key name. Use instead of `tap`. |
| `label` | `{ text = "<div>", label = "div" }` | What the key shows. |
| `width` | `{ tap = "<space>", width = 3.0 }` | Relative width; a normal key is `1.0`. |
| `hold` | `{ tap = ".", hold = ".." }` | What a long-press does, in key notation. Shown as a small hint on the key. |
| `swipe` | `{ tap = "<space>", swipe = "cursor" }` | Drag sideways to move the cursor; with Shift on, it selects. |
| `repeat` | `{ tap = "<bksp>", repeat = false }` | Repeat while held. On by default for Backspace, Delete, arrows, Page Up and Page Down. |

A key does one thing while held: it repeats, or it has a `hold`, or it swipes. An
explicit `repeat = true` together with `hold` or `swipe` is an error; the default repeat
of Backspace and the arrows quietly gives way.

Layouts are data: they can insert text and press keys, never run code.
