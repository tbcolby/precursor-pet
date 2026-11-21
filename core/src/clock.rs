//! Turning two imperfect clocks into "seconds since I last looked".
//!
//! * The wall clock (RTC through `llio`) keeps counting through suspend and
//!   reboots, but may be unset, and can be changed by the user.
//! * The monotonic ticktimer never jumps, but stops while the device is
//!   suspended and restarts at boot.
//!
//! Policy: use the wall clock whenever it's set and hasn't gone backwards;
//! otherwise use the monotonic delta. When the wall clock disappears its last
//! reading is forgotten, so reappearing later can't double-count time already
//! counted from the monotonic clock.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Timekeeper {
    last_wall_s: Option<u64>,
    last_mono_ms: Option<u64>,
}

impl Timekeeper {
    /// Resume from a saved wall-clock reading (`None` = never saved or unknown).
    pub fn from_saved(last_wall_s: Option<u64>) -> Self { Self { last_wall_s, last_mono_ms: None } }

    /// The wall reading to store with a snapshot of the pet.
    pub fn last_wall_s(&self) -> Option<u64> { self.last_wall_s }

    /// Seconds elapsed since the previous call. `wall_s` is the wall clock in
    /// seconds if it is set; `mono_ms` is the ticktimer reading.
    pub fn elapsed(&mut self, wall_s: Option<u64>, mono_ms: u64) -> u64 {
        let mono_delta = self.last_mono_ms.map(|m| mono_ms.saturating_sub(m) / 1000);
        let e = match (wall_s, self.last_wall_s) {
            (Some(w), Some(l)) if w >= l => w - l,
            // Clock set back: trust the monotonic clock, if we have one.
            (Some(_), Some(_)) => mono_delta.unwrap_or(0),
            // First wall reading after none (or after boot): baseline only.
            _ => mono_delta.unwrap_or(0),
        };
        // Only whole elapsed seconds advance the baselines, so sub-second
        // remainders aren't lost across frequent calls.
        match wall_s {
            Some(w) => {
                self.last_wall_s = Some(w);
            }
            None => self.last_wall_s = None,
        }
        self.last_mono_ms = Some(mono_ms);
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_clock_covers_suspend() {
        let mut t = Timekeeper::default();
        t.elapsed(Some(1000), 5_000);
        // ten minutes asleep: ticktimer moved 2 s, wall clock 600 s
        assert_eq!(t.elapsed(Some(1600), 7_000), 600);
    }

    #[test]
    fn saved_wall_reading_covers_reboot() {
        let mut t = Timekeeper::from_saved(Some(1_000));
        // new boot: ticktimer starts near zero, no previous mono reading
        assert_eq!(t.elapsed(Some(4_600), 3_000), 3_600);
        assert_eq!(t.elapsed(Some(4_660), 63_000), 60);
    }

    #[test]
    fn unset_clock_falls_back_to_monotonic() {
        let mut t = Timekeeper::default();
        assert_eq!(t.elapsed(None, 10_000), 0);
        assert_eq!(t.elapsed(None, 70_000), 60);
        assert_eq!(t.last_wall_s(), None);
    }

    #[test]
    fn clock_appearing_later_does_not_double_count() {
        // Saved wall reading from yesterday, but the clock is unset this boot.
        let mut t = Timekeeper::from_saved(Some(1_000));
        assert_eq!(t.elapsed(None, 1_000), 0, "unset: no wall delta");
        assert_eq!(t.elapsed(None, 61_000), 60);
        // User sets the clock to something days later: baseline only, mono delta counts.
        assert_eq!(t.elapsed(Some(900_000), 121_000), 60);
        assert_eq!(t.elapsed(Some(900_060), 181_000), 60);
    }

    #[test]
    fn clock_set_back_uses_monotonic() {
        let mut t = Timekeeper::default();
        t.elapsed(Some(100_000), 0);
        assert_eq!(t.elapsed(Some(50_000), 30_000), 30);
        // and recovers: next reading is relative to the new time
        assert_eq!(t.elapsed(Some(50_060), 90_000), 60);
    }

    #[test]
    fn wall_clock_set_back_without_mono_is_zero() {
        let mut t = Timekeeper::from_saved(Some(100_000));
        assert_eq!(t.elapsed(Some(10), 0), 0);
    }

    #[test]
    fn mono_going_backwards_is_harmless() {
        let mut t = Timekeeper::default();
        t.elapsed(None, 100_000);
        assert_eq!(t.elapsed(None, 5_000), 0);
    }
}
