# typing-coach

A playful terminal typing trainer for the **German QWERTZ** layout. It takes you from a total
beginner on `f` and `j` to a fast typist, with an optional track for programming symbols like
`{ } ( ) [ ]`.

## Features

- **Step-by-step lessons:** starts on the home row with `f j`, adds two keys at a time
  (`d k`, `s l`, `a ö`, … `t z`, `ü ä`, `y -`, `ß`), then capitals, numbers and punctuation.
  Each lesson only uses keys you already know plus the new ones.
- **On-screen QWERTZ keyboard:** the next key lights up in the color of the finger that should
  press it, with hints for Shift (opposite hand) and AltGr (right thumb).
- **Accuracy first:** a wrong key doesn't move the cursor; you must type the right one.
- **Day by day:** at most 2 new lessons per day by default (adjustable), because motor skills
  consolidate during sleep.
- **Daily 2-minute training:** mixes everything you've learned and shows your weakest keys
  (most errors, slowest) more often. Keeps a daily streak.
- **Speed levels:** after the lessons, levels start at 25 WPM and rise in 5 WPM steps.
- **Programming track (optional):** `( )`, `{ }`, `[ ]`, `= + *`, `< >`, `/ \ |`, `_ #`,
  `& % $`, `@ ~` with real code snippets, followed by its own speed levels.
- **Statistics:** WPM history, an error heatmap on the keyboard and your weakest keys.

## Install

### Arch Linux / Omarchy: prebuilt package (x86_64)

Download `typing-coach-<version>-x86_64.pkg.tar.zst` from the
[latest release](https://github.com/stikkx/typing-coach/releases/latest), then:

```sh
sudo pacman -U typing-coach-*-x86_64.pkg.tar.zst
```

### Arch Linux / Omarchy: build the package yourself

```sh
git clone https://github.com/stikkx/typing-coach.git
cd typing-coach/pkg/arch
makepkg -si
```

### Any system with Rust (edition 2024)

```sh
cargo install --git https://github.com/stikkx/typing-coach --tag v0.1.0
```

Then start it with `typing_coach` (the Arch package also installs it as `typing-coach`).
To run it straight from a source checkout, use `cargo run --release`.

Set your system keyboard layout to German (`de`). The app reads the characters your layout
produces, so AltGr symbols like `{` work as usual.

## Progress

Progress is saved to `~/.local/share/typing_coach/progress.json`. Set `TYPING_COACH_FILE`
to use a different file.

## Keys

| Screen | Keys |
|---|---|
| Menu | `↑`/`↓` select, `Enter` start, `1`–`9` quick pick, `q` quit |
| Typing | just type, `Esc` abort |
| Result | `Enter` back to menu, `r` go again |

## License

MIT
