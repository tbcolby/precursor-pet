# precursor-pet

A virtual pet for [Precursor](https://precursor.dev) that lives in real time. It gets hungry, bored, tired and dirty over hours, not seconds, and it keeps living while the app is in the background, the device sleeps, or it is switched off.

## How it works

- **Real time.** The pet is advanced by the seconds that actually passed. Opening the app (or returning to it) asks the clocks how long you were away and applies that in one go; there is no background ticking. The result is identical to having watched it the whole time.
- **Clocks.** The wall clock (RTC) is used when it is set, so suspend and reboots count. If the clock is unset, or was set backwards, the monotonic timer is used instead, which covers backgrounding but not suspend or reboot; the pet screen warns "Clock not set" in that case. Catch-up is capped at 14 days.
- **Neglect while away.** If more than 10 minutes pass with the app not running, the pet can get hungry, dirty and sick but cannot die of neglect: its health bottoms out at 1. It can die of neglect only while you are looking at it, or of old age at any time.
- **Pace.** Hunger rises about 1 point per 7 minutes (full to starving in roughly 12 hours), happiness falls about 1 per 9, energy 1 per 10, a poop every 3 hours. A pet left unfed on screen dies roughly 10 to 15 hours later.
- **Lifecycle.** Egg (hatches after 5 minutes), baby (to 12 hours), child (to 3 days), teen (to 7 days), adult (to 14 days), senior (to 21 days), then it dies of old age. The longest life is remembered.
- **Care.** Feed, play, sleep/wake (a sleeping pet regains energy and wakes by itself; it can't be fed or played with), medicine (cures sickness, which starts when health falls below 40 and only medicine clears), clean up (3 or more poops hurt health), discipline. Heavy pets (weight over 60) lose health; an undisciplined pet (discipline under 30) gets bored twice as fast, and discipline fades by 1 per hour.
- When the pet dies, Enter hatches a new egg.

## Keys

| Screen | Keys |
|---|---|
| Naming | Type a name (16 characters), Left/Right move, Backspace deletes (on an empty name it leaves), Enter hatches, F4 leaves |
| Pet | Enter/Right/Down open the care menu, `1`-`6` act directly, Backspace or F4 leave; Enter hatches a new egg after a death |
| Care menu | Up/Down pick, Enter act, `1`-`6` act, Left, Backspace or F4 back |

The menu key (centre of the D-pad) is consumed by Xous and never reaches the app.

## Storage

PDDB dictionary `pet`, key `pet`: one versioned JSON blob with the pet, the wall-clock reading of the same moment, and the longest life. It is written after each care action, when the app goes to the background or leaves, and only when something changed. Data that can't be read or comes from a newer version is never overwritten (the app then runs without saving). Reloaded when a basis is locked or unlocked, unless you have an unsaved care action.

## Limitations

- The time rules are a design choice, not Tamagotchi canon; rates are constants in `core/src/pet.rs`.
- Time away is only as good as the clock: without a set RTC, suspend and reboot time is not counted.
- There are no sounds or vibration alerts; the pet can't call for you while the app is closed.
- Layout (sprite and bars) hasn't been checked on a device. Renode capture not yet run: `capture.py` is written but not executed; verification so far is the hosted build, clippy, lint and host tests.
- An earlier version had no persistence, so there is nothing to migrate.

## Building

Builds inside a [xous-core](https://github.com/betrusted-io/xous-core) tree. With [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit):

```bash
bin/integrate --xous-root ~/xous-core --grade ~/Dev/precursor/precursor-pet
```

Host tests alone: `cd core && cargo test`.

## Layout

```
core/src/pet.rs      needs, lifecycle, actions, save format
core/src/clock.rs    wall/monotonic clocks to elapsed seconds
core/tests/pet.rs    time-model, action and save tests
src/main.rs, app.rs  GAM registration, key mapping, drawing, saving
capture.py           Renode capture sequence (not yet run)
```

## Author

Tyler Colby, [Colby's Data Movers, LLC](https://colbysdatamovers.com). Licensed under the Apache License, Version 2.0 (see [LICENSE](LICENSE)).
