//! The pet: needs, lifecycle and actions, all driven by elapsed *real* time.
//!
//! Time model: the pet lives in one-minute steps ([`TICK_S`]). The shell calls
//! [`Pet::advance`] with the seconds that really passed (see
//! `clock::Timekeeper`), whether that's a 20-second frame or a night in a
//! pocket, so nothing is ticked by a timer and nothing needs the app to be in
//! front. The result is the same however the elapsed time is chunked.
//!
//! All numbers are `u8` or `u64` with saturating arithmetic.

use serde::{Deserialize, Serialize};

pub const TICK_S: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 24 * HOUR;

/// Longest stretch a single `advance` simulates (14 days). Anything longer
/// is treated as 14 days.
pub const MAX_CATCHUP_S: u64 = 14 * DAY;
/// Elapsed time beyond this counts as "away": the pet may get worse but
/// can't die of neglect while the app wasn't running.
pub const AWAY_AFTER_S: u64 = 10 * 60;

/// Age at which each stage begins.
pub const HATCH_S: u64 = 5 * 60;
pub const CHILD_S: u64 = 12 * HOUR;
pub const TEEN_S: u64 = 3 * DAY;
pub const ADULT_S: u64 = 7 * DAY;
pub const SENIOR_S: u64 = 14 * DAY;
/// Natural lifespan: the pet dies of old age at this age.
pub const LIFESPAN_S: u64 = 21 * DAY;

pub const NAME_MAX: usize = 16;
pub const SAVE_VERSION: u32 = 1;
const MAX_EVENTS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    Egg,
    Baby,
    Child,
    Teen,
    Adult,
    Senior,
}

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Egg => "Egg",
            Stage::Baby => "Baby",
            Stage::Child => "Child",
            Stage::Teen => "Teen",
            Stage::Adult => "Adult",
            Stage::Senior => "Senior",
        }
    }

    pub fn of_age(age_s: u64) -> Stage {
        match age_s {
            a if a < HATCH_S => Stage::Egg,
            a if a < CHILD_S => Stage::Baby,
            a if a < TEEN_S => Stage::Child,
            a if a < ADULT_S => Stage::Teen,
            a if a < SENIOR_S => Stage::Adult,
            _ => Stage::Senior,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Happy,
    Content,
    Bored,
    Sad,
    Sick,
    Asleep,
}

impl Mood {
    pub fn name(self) -> &'static str {
        match self {
            Mood::Happy => "Happy",
            Mood::Content => "Content",
            Mood::Bored => "Bored",
            Mood::Sad => "Sad",
            Mood::Sick => "Sick",
            Mood::Asleep => "Asleep",
        }
    }

    pub fn face(self) -> &'static str {
        match self {
            Mood::Happy => ":D",
            Mood::Content => ":)",
            Mood::Bored => ":/",
            Mood::Sad => ":(",
            Mood::Sick => "x(",
            Mood::Asleep => "-_-",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cause {
    Neglect,
    OldAge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    /// Seconds lived (stops at death).
    pub age_s: u64,
    /// 0 = full, 100 = starving.
    pub hunger: u8,
    pub happiness: u8,
    pub energy: u8,
    /// 0 = dead.
    pub health: u8,
    pub weight: u8,
    pub discipline: u8,
    pub poop: u8,
    pub sick: bool,
    pub sleeping: bool,
    pub alive: bool,
    pub cause: Option<Cause>,
    /// Whole minutes simulated; the phase for every periodic effect.
    t: u64,
    /// Seconds not yet making a whole minute.
    carry_s: u64,
}

impl Pet {
    pub fn new(name: &str) -> Pet {
        let name: String = name.trim().chars().filter(|c| !c.is_control()).take(NAME_MAX).collect();
        Pet {
            name: if name.is_empty() { "Pixel".to_string() } else { name },
            age_s: 0,
            hunger: 20,
            happiness: 80,
            energy: 100,
            health: 100,
            weight: 5,
            discipline: 50,
            poop: 0,
            sick: false,
            sleeping: false,
            alive: true,
            cause: None,
            t: 0,
            carry_s: 0,
        }
    }

    pub fn stage(&self) -> Stage { Stage::of_age(self.age_s) }

    pub fn mood(&self) -> Mood {
        if self.sleeping {
            Mood::Asleep
        } else if self.sick {
            Mood::Sick
        } else if self.happiness >= 80 && self.hunger < 30 {
            Mood::Happy
        } else if self.happiness >= 50 {
            Mood::Content
        } else if self.happiness >= 30 {
            Mood::Bored
        } else {
            Mood::Sad
        }
    }

    /// Let `elapsed_s` real seconds pass. Returns notable events (at most 8,
    /// newest last). `away` = the time passed while the app wasn't running
    /// (or the device slept): the pet can't die of neglect then, it bottoms
    /// out at 1 health; old age still applies.
    pub fn advance(&mut self, elapsed_s: u64, away: bool) -> Vec<String> {
        let mut events = Vec::new();
        if !self.alive {
            return events;
        }
        let total = self.carry_s.saturating_add(elapsed_s.min(MAX_CATCHUP_S));
        let ticks = total / TICK_S;
        self.carry_s = total % TICK_S;
        for _ in 0..ticks {
            self.step_minute(away, &mut events);
            if !self.alive {
                break;
            }
        }
        if events.len() > MAX_EVENTS {
            events.drain(..events.len() - MAX_EVENTS);
        }
        events
    }

    /// `advance` with `away` derived from how much time passed.
    pub fn advance_auto(&mut self, elapsed_s: u64) -> Vec<String> { self.advance(elapsed_s, elapsed_s > AWAY_AFTER_S) }

    fn step_minute(&mut self, away: bool, ev: &mut Vec<String>) {
        let before = self.stage();
        self.t = self.t.saturating_add(1);
        self.age_s = self.age_s.saturating_add(TICK_S);
        let after = self.stage();
        if after != before {
            ev.push(match after {
                Stage::Baby => format!("{} hatched!", self.name),
                Stage::Child => format!("{} grew into a child.", self.name),
                Stage::Teen => format!("{} is a teenager now.", self.name),
                Stage::Adult => format!("{} is an adult.", self.name),
                _ => format!("{} is getting old.", self.name),
            });
        }
        if self.age_s >= LIFESPAN_S {
            self.die(Cause::OldAge, ev);
            return;
        }
        if after == Stage::Egg {
            return;
        }
        let t = self.t;
        if self.sleeping {
            self.energy = self.energy.saturating_add(2).min(100);
            if t % 14 == 0 {
                self.hunger = self.hunger.saturating_add(1).min(100);
            }
            if self.energy == 100 {
                self.sleeping = false;
                ev.push(format!("{} woke up.", self.name));
            }
        } else {
            if t % 7 == 0 {
                self.hunger = self.hunger.saturating_add(1).min(100);
            }
            // An undisciplined pet gets bored faster.
            let boredom = if self.discipline < 30 { 4 } else { 9 };
            if t % boredom == 0 {
                self.happiness = self.happiness.saturating_sub(1);
            }
            if t % 10 == 0 {
                self.energy = self.energy.saturating_sub(1);
            }
        }
        if t % 180 == 0 && self.poop < 5 {
            self.poop += 1;
        }
        if t % 60 == 0 {
            self.discipline = self.discipline.saturating_sub(1);
        }

        let mut dmg = 0u8;
        if self.hunger > 80 && t % 3 == 0 {
            dmg += 1;
        }
        if self.poop >= 3 && t % 10 == 0 {
            dmg += 1;
        }
        if self.happiness < 20 && t % 10 == 0 {
            dmg += 1;
        }
        if self.sick && t % 6 == 0 {
            dmg += 1;
        }
        if self.weight > 60 && t % 20 == 0 {
            dmg += 1;
        }
        if self.energy == 0 && t % 10 == 0 {
            dmg += 1;
        }
        self.health = self.health.saturating_sub(dmg);
        if dmg == 0 && self.hunger < 50 && self.happiness >= 40 && self.poop < 3 && !self.sick && self.weight <= 60 && t % 5 == 0 {
            self.health = self.health.saturating_add(1).min(100);
        }
        if self.health < 40 && !self.sick {
            self.sick = true;
            ev.push(format!("{} is sick!", self.name));
        }
        if self.health == 0 {
            if away {
                self.health = 1;
            } else {
                self.die(Cause::Neglect, ev);
            }
        }
    }

    fn die(&mut self, cause: Cause, ev: &mut Vec<String>) {
        self.alive = false;
        self.cause = Some(cause);
        self.sleeping = false;
        if cause == Cause::Neglect {
            self.health = 0;
        }
        ev.push(match cause {
            Cause::Neglect => format!("{} has passed away...", self.name),
            Cause::OldAge => format!("{} died peacefully of old age.", self.name),
        });
    }

    // ---- actions ----

    fn blocked(&self) -> Option<&'static str> {
        if !self.alive || self.stage() == Stage::Egg {
            Some("...")
        } else if self.sleeping {
            Some("Zzz... (wake it first)")
        } else {
            None
        }
    }

    pub fn feed(&mut self) -> &'static str {
        if let Some(m) = self.blocked() {
            return m;
        }
        if self.hunger < 10 {
            return "Not hungry!";
        }
        self.hunger = self.hunger.saturating_sub(30);
        self.weight = self.weight.saturating_add(1).min(99);
        "Yum!"
    }

    pub fn play(&mut self) -> &'static str {
        if let Some(m) = self.blocked() {
            return m;
        }
        if self.energy < 10 {
            return "Too tired!";
        }
        self.happiness = self.happiness.saturating_add(25).min(100);
        self.energy = self.energy.saturating_sub(15);
        self.hunger = self.hunger.saturating_add(5).min(100);
        self.weight = self.weight.saturating_sub(1).max(1);
        "Wheee!"
    }

    /// Toggle sleep. A sleeping pet recovers energy until it wakes by itself.
    pub fn sleep(&mut self) -> &'static str {
        if !self.alive || self.stage() == Stage::Egg {
            return "...";
        }
        if self.sleeping {
            self.sleeping = false;
            "Awake."
        } else if self.energy >= 100 {
            "Not sleepy."
        } else {
            self.sleeping = true;
            "Zzz..."
        }
    }

    pub fn medicine(&mut self) -> &'static str {
        if !self.alive || self.stage() == Stage::Egg {
            return "...";
        }
        if !self.sick {
            return "Not sick!";
        }
        self.sick = false;
        self.health = self.health.saturating_add(30).min(100);
        self.happiness = self.happiness.saturating_sub(10);
        "Medicine given."
    }

    pub fn clean(&mut self) -> &'static str {
        if !self.alive || self.stage() == Stage::Egg {
            return "...";
        }
        if self.poop == 0 {
            return "Already clean!";
        }
        self.poop = 0;
        "Cleaned up!"
    }

    pub fn discipline(&mut self) -> &'static str {
        if let Some(m) = self.blocked() {
            return m;
        }
        self.discipline = self.discipline.saturating_add(10).min(100);
        self.happiness = self.happiness.saturating_sub(5);
        "Disciplined."
    }

    pub fn sprite(&self) -> &'static [&'static str] {
        match self.stage() {
            Stage::Egg => &["  ___  ", " /   \\ ", "|  o  |", " \\___/ "],
            Stage::Baby => &["  ^_^  ", " /o o\\ ", "|  w  |", "  \\_/  "],
            Stage::Child => &["  /^\\  ", " (o o) ", " ( w ) ", " /| |\\ "],
            Stage::Teen => &["  /^\\  ", " (O O) ", " (=w=) ", " /| |\\ ", "  d b  "],
            Stage::Adult => &[" \\^_^/ ", " (O_O) ", "[(=w=)]", " /| |\\ ", " _d b_ "],
            Stage::Senior => &["  ___  ", " (- -) ", " (=w=) ", " /| |\\ ", "  d b  "],
        }
    }

    /// Human-readable age, e.g. "2d 3h" or "14m".
    pub fn age_label(&self) -> String {
        let a = self.age_s;
        if a >= DAY {
            format!("{}d {}h", a / DAY, a % DAY / HOUR)
        } else if a >= HOUR {
            format!("{}h {}m", a / HOUR, a % HOUR / 60)
        } else {
            format!("{}m", a / 60)
        }
    }
}

// ---- save format ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Saved {
    pub version: u32,
    pub pet: Option<Pet>,
    /// Wall clock (seconds) when this snapshot was taken, if the clock was set.
    pub wall_s: Option<u64>,
    /// Longest life so far, seconds.
    pub best_age_s: u64,
}

impl Default for Saved {
    fn default() -> Self { Saved { version: SAVE_VERSION, pet: None, wall_s: None, best_age_s: 0 } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    NewerVersion(u32),
    Corrupt,
}

impl Saved {
    pub fn check(&self) -> Result<(), LoadError> {
        if self.version > SAVE_VERSION {
            return Err(LoadError::NewerVersion(self.version));
        }
        if let Some(p) = &self.pet {
            let stats = [p.hunger, p.happiness, p.energy, p.health, p.discipline];
            let bad = p.name.is_empty()
                || p.name.chars().count() > NAME_MAX
                || p.name.chars().any(|c| c.is_control())
                || stats.iter().any(|v| *v > 100)
                || p.weight == 0
                || p.weight > 99
                || p.poop > 5
                || p.carry_s >= TICK_S
                || (p.alive && p.health == 0)
                || (!p.alive && p.cause.is_none())
                || p.age_s > LIFESPAN_S + TICK_S;
            if bad {
                return Err(LoadError::Corrupt);
            }
        }
        Ok(())
    }
}
