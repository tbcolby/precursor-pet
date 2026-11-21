# Pet build notes

- Time model (`core/src/pet.rs`, `core/src/clock.rs`): the pet lives in one-minute steps and is advanced by *elapsed real seconds*, never by a timer. `Timekeeper::elapsed(wall_s, mono_ms)` turns the RTC (`llio::LocalTime`) and ticktimer into elapsed seconds (wall preferred; monotonic fallback when the clock is unset or set backwards). Elapsed time over 10 minutes counts as "away": the pet can drop to 1 health but not die of neglect; old age always applies. Catch-up is capped at 14 days.
- Persistence stores the pet and the wall-clock reading of the same snapshot (`Saved`), so a reboot catches up from exactly where the stored state left off. Sync before saving.
- The shell pumps only to refresh the screen while in front (30 s); nothing runs in the background, and the foreground sync catches up.
- Tests: `cd core && cargo test` (chunking independence, catch-up equals live ticking, away floor, clock edge cases, save validation).
- Backspace edits the name on the naming screen (empty name: leaves); elsewhere Backspace/F4 leave or go back. `∴` is never delivered. Leave with `kit::leave`, never `terminate_process`.
