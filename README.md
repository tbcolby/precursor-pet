# Tamagotchi

A virtual pet (Tamagotchi-style) for the [Precursor](https://precursor.dev) handheld, with lifecycle stages, stat bars, ASCII art sprites, and 6 care actions on the 336x536 monochrome 1-bit display.

## Features

- **6 lifecycle stages** — Egg, Baby, Child, Teen, Adult, Senior
- **5 moods** — Happy, Content, Bored, Sad, Sick with face emoticons
- **5 stat bars** — Hunger, Happiness, Energy, Health, Discipline
- **6 actions** — Feed, Play, Sleep, Medicine, Clean, Discipline
- **ASCII art sprites** — Different art for each lifecycle stage
- **Poop system** — Accumulates over time, needs cleaning
- **Sickness & death** — Neglect leads to illness and eventual death
- **Name your pet** — Custom naming at start

## Architecture

- `pet.rs` — Pet struct, lifecycle stages, stat decay, actions, ASCII sprites
- `app.rs` — 3-state machine (Naming → Status ↔ Action)
- `ui.rs` — Pet display with stat bars, action menu, message overlay
- `main.rs` — Xous event loop with 3-second tick timer

## Controls

| Key | Action |
|-----|--------|
| Type | Name your pet |
| Enter | Hatch / Perform action |
| Right | Open actions |
| Up/Down | Browse actions |
| Left | Back to status |
| Menu (∴) | Quit |

---

## Development

This app was developed using the methodology described in [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) — an LLM-assisted approach to Precursor app development on macOS ARM64.

---

## Author

Made by Tyler Colby — [Colby's Data Movers, LLC](https://colbysdatamovers.com)

Contact: [tyler@colbysdatamovers.com](mailto:tyler@colbysdatamovers.com) | [GitHub Issues](https://github.com/tbcolby/precursor-pet/issues)

---

## License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for the full text.
