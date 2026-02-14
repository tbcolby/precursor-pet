//! Virtual pet simulation — stats, actions, lifecycle.

extern crate alloc;
use alloc::string::String;
use alloc::format;

/// Pet lifecycle stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stage { Egg, Baby, Child, Teen, Adult, Senior }

impl Stage {
    pub fn name(&self) -> &'static str {
        match self { Stage::Egg => "Egg", Stage::Baby => "Baby", Stage::Child => "Child", Stage::Teen => "Teen", Stage::Adult => "Adult", Stage::Senior => "Senior" }
    }
}

/// Pet mood.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood { Happy, Content, Bored, Sad, Sick }

impl Mood {
    pub fn name(&self) -> &'static str {
        match self { Mood::Happy => "Happy", Mood::Content => "Content", Mood::Bored => "Bored", Mood::Sad => "Sad", Mood::Sick => "Sick" }
    }
    pub fn face(&self) -> &'static str {
        match self { Mood::Happy => ":D", Mood::Content => ":)", Mood::Bored => ":/", Mood::Sad => ":(", Mood::Sick => "x(" }
    }
}

/// The virtual pet.
pub struct Pet {
    pub name: String,
    pub stage: Stage,
    pub mood: Mood,
    pub hunger: u8,      // 0=full, 100=starving
    pub happiness: u8,   // 0=miserable, 100=ecstatic
    pub energy: u8,      // 0=exhausted, 100=energetic
    pub health: u8,      // 0=dead, 100=perfect
    pub age: u32,        // in ticks
    pub weight: u8,      // 1-255
    pub discipline: u8,  // 0-100
    pub alive: bool,
    pub poop_count: u8,
    pub sick_ticks: u8,
}

impl Pet {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            stage: Stage::Egg,
            mood: Mood::Content,
            hunger: 20,
            happiness: 80,
            energy: 100,
            health: 100,
            age: 0,
            weight: 5,
            discipline: 50,
            alive: true,
            poop_count: 0,
            sick_ticks: 0,
        }
    }

    /// Advance one tick (called periodically).
    pub fn tick(&mut self) -> Option<String> {
        if !self.alive { return None; }
        self.age += 1;
        let mut msg: Option<String> = None;

        // Stage evolution
        let new_stage = match self.stage {
            Stage::Egg if self.age >= 5 => { msg = Some(format!("{} hatched!", self.name)); Some(Stage::Baby) }
            Stage::Baby if self.age >= 30 => { msg = Some(format!("{} grew into a Child!", self.name)); Some(Stage::Child) }
            Stage::Child if self.age >= 80 => { msg = Some(format!("{} is now a Teen!", self.name)); Some(Stage::Teen) }
            Stage::Teen if self.age >= 150 => { msg = Some(format!("{} is now an Adult!", self.name)); Some(Stage::Adult) }
            Stage::Adult if self.age >= 300 => { msg = Some(format!("{} is now a Senior.", self.name)); Some(Stage::Senior) }
            _ => None,
        };
        if let Some(s) = new_stage { self.stage = s; }

        // Stat decay
        if self.stage != Stage::Egg {
            if self.hunger < 100 { self.hunger = (self.hunger + 2).min(100); }
            if self.happiness > 0 { self.happiness = self.happiness.saturating_sub(1); }
            if self.energy > 0 { self.energy = self.energy.saturating_sub(1); }

            // Poop every ~20 ticks
            if self.age % 20 == 0 && self.poop_count < 5 { self.poop_count += 1; }

            // Health effects
            if self.hunger > 80 { self.health = self.health.saturating_sub(2); }
            if self.poop_count >= 3 { self.health = self.health.saturating_sub(1); }
            if self.happiness < 20 { self.health = self.health.saturating_sub(1); }

            // Sickness
            if self.health < 40 && self.sick_ticks == 0 { self.sick_ticks = 1; }
            if self.sick_ticks > 0 { self.sick_ticks += 1; self.health = self.health.saturating_sub(1); }
            if self.sick_ticks > 20 && self.health < 10 { self.alive = false; msg = Some(format!("{} has passed away...", self.name)); }

            // Death from starvation
            if self.hunger >= 100 && self.health == 0 { self.alive = false; msg = Some(format!("{} starved...", self.name)); }
        }

        self.update_mood();
        msg
    }

    fn update_mood(&mut self) {
        if !self.alive { return; }
        if self.sick_ticks > 0 { self.mood = Mood::Sick; }
        else if self.happiness >= 80 && self.hunger < 30 { self.mood = Mood::Happy; }
        else if self.happiness >= 50 { self.mood = Mood::Content; }
        else if self.happiness >= 30 { self.mood = Mood::Bored; }
        else { self.mood = Mood::Sad; }
    }

    /// Feed the pet.
    pub fn feed(&mut self) -> &'static str {
        if !self.alive || self.stage == Stage::Egg { return "..."; }
        if self.hunger < 10 { return "Not hungry!"; }
        self.hunger = self.hunger.saturating_sub(30);
        self.weight = self.weight.saturating_add(1);
        self.update_mood();
        "Yum!"
    }

    /// Play with the pet.
    pub fn play(&mut self) -> &'static str {
        if !self.alive || self.stage == Stage::Egg { return "..."; }
        if self.energy < 10 { return "Too tired!"; }
        self.happiness = (self.happiness + 25).min(100);
        self.energy = self.energy.saturating_sub(15);
        self.hunger = (self.hunger + 5).min(100);
        if self.weight > 1 { self.weight -= 1; }
        self.update_mood();
        "Wheee!"
    }

    /// Put the pet to sleep.
    pub fn sleep(&mut self) -> &'static str {
        if !self.alive || self.stage == Stage::Egg { return "..."; }
        self.energy = (self.energy + 40).min(100);
        self.update_mood();
        "Zzz..."
    }

    /// Give medicine.
    pub fn medicine(&mut self) -> &'static str {
        if !self.alive || self.stage == Stage::Egg { return "..."; }
        if self.sick_ticks == 0 { return "Not sick!"; }
        self.sick_ticks = 0;
        self.health = (self.health + 30).min(100);
        self.happiness = self.happiness.saturating_sub(10); // doesn't like medicine
        self.update_mood();
        "Medicine given."
    }

    /// Clean up poop.
    pub fn clean(&mut self) -> &'static str {
        if self.poop_count == 0 { return "Already clean!"; }
        self.poop_count = 0;
        "Cleaned up!"
    }

    /// Discipline the pet.
    pub fn discipline(&mut self) -> &'static str {
        if !self.alive || self.stage == Stage::Egg { return "..."; }
        self.discipline = (self.discipline + 10).min(100);
        self.happiness = self.happiness.saturating_sub(5);
        self.update_mood();
        "Disciplined."
    }

    /// ASCII art for current stage.
    pub fn sprite(&self) -> &'static [&'static str] {
        match self.stage {
            Stage::Egg => &["  ___  ", " /   \\ ", "|  o  |", " \\___/ "],
            Stage::Baby => &["  ^_^  ", " /o o\\ ", "|  w  |", "  \\_/  "],
            Stage::Child => &["  /^\\  ", " (o o) ", " ( w ) ", " /| |\\ "],
            Stage::Teen => &["  /^\\  ", " (O O) ", " (=w=) ", " /| |\\ ", "  d b  "],
            Stage::Adult => &[" \\^_^/ ", " (O_O) ", "[(=w=)]", " /| |\\ ", " _d b_ "],
            Stage::Senior => &["  ___  ", " (- -) ", " (=w=) ", " /| |\\ ", "  d b  "],
        }
    }
}
