//! Input, saving and drawing for the pet. The time model and all rules live
//! in `pet-core`.

use gam::menu::{PixelColor, Point};
use gam::GlyphStyle;
use pet_core::{Pet, Saved, Stage, TextField, Timekeeper, AWAY_AFTER_S, NAME_MAX};
use precursor_appkit::keys::Key;
use precursor_appkit::layout::ListState;
use precursor_appkit::screen::{Screen, MARGIN};
use precursor_appkit::store::Store;

const STATE_KEY: &str = "pet";

const ACTIONS: [&str; 6] = ["Feed", "Play", "Sleep / wake", "Medicine", "Clean up", "Discipline"];

#[derive(PartialEq, Eq)]
pub enum Action {
    Stay,
    Leave,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Naming,
    Status,
    Actions,
}

pub struct App {
    screen: Screen,
    store: Store,
    tt: ticktimer_server::Ticktimer,
    wall: llio::LocalTime,
    clock: Timekeeper,
    pet: Option<Pet>,
    best_age_s: u64,
    page: Page,
    name: TextField,
    list: ListState,
    status: String,
    /// Something changed since the last save.
    dirty: bool,
    /// The player did something (care action or new pet), as opposed to time just passing.
    user_dirty: bool,
    /// Stored data couldn't be used; never overwrite it.
    read_only: bool,
}

fn fmt_duration(s: u64) -> String {
    if s >= 86_400 {
        format!("{}d {}h", s / 86_400, s % 86_400 / 3600)
    } else if s >= 3600 {
        format!("{}h {}m", s / 3600, s % 3600 / 60)
    } else {
        format!("{}m", s / 60)
    }
}

impl App {
    pub fn new(screen: Screen, store: Store, _xns: &xous_names::XousNames) -> Self {
        let mut app = Self {
            screen,
            store,
            tt: ticktimer_server::Ticktimer::new().expect("pet: no ticktimer"),
            wall: llio::LocalTime::new(),
            clock: Timekeeper::default(),
            pet: None,
            best_age_s: 0,
            page: Page::Naming,
            name: TextField::with_max(NAME_MAX),
            list: ListState::default(),
            status: String::new(),
            dirty: false,
            user_dirty: false,
            read_only: false,
        };
        app.reload();
        app
    }

    pub fn screen(&self) -> &Screen { &self.screen }

    fn wall_s(&mut self) -> Option<u64> { self.wall.get_local_time_ms().map(|ms| ms / 1000) }

    /// Apply the real time that has passed since the last look.
    fn sync(&mut self) {
        let wall = self.wall_s();
        let e = self.clock.elapsed_secs(wall, self.tt.elapsed_ms());
        let mut best = self.best_age_s;
        if let Some(p) = self.pet.as_mut().filter(|p| p.alive) {
            let events = p.advance_auto(e);
            if e > 0 {
                self.dirty = true;
            }
            if let Some(last) = events.last() {
                self.status = last.clone();
            } else if e > AWAY_AFTER_S {
                self.status = format!("Welcome back! (away {})", fmt_duration(e));
            }
            if !p.alive {
                best = best.max(p.age_s);
            }
        }
        if best != self.best_age_s {
            self.best_age_s = best;
            self.dirty = true;
        }
    }

    /// Load at start-up and whenever the open bases change. A player action
    /// that hasn't been saved yet is never thrown away.
    pub fn reload(&mut self) {
        if self.user_dirty {
            return;
        }
        self.read_only = false;
        match self.store.load::<Saved>(STATE_KEY) {
            Ok(Some(s)) => match s.check() {
                Ok(()) => {
                    self.best_age_s = s.best_age_s;
                    self.clock = Timekeeper::from_saved(s.wall_s);
                    self.page = if s.pet.is_some() { Page::Status } else { Page::Naming };
                    self.pet = s.pet;
                    self.status.clear();
                    self.dirty = false;
                    self.sync();
                }
                Err(e) => {
                    log::warn!("pet: save unusable: {:?}", e);
                    self.read_only = true;
                    self.status = "Saved pet unreadable; playing without saving".into();
                }
            },
            Ok(None) => {
                self.clock = Timekeeper::default();
                self.sync();
            }
            Err(e) => {
                log::error!("pet: couldn't load: {:?}", e.kind());
                self.read_only = true;
                self.status = "Couldn't read saved pet; playing without saving".into();
            }
        }
        self.draw();
    }

    fn save_if_dirty(&mut self) {
        if !self.dirty || self.read_only {
            return;
        }
        // Bring the pet up to date first, so the stored state and the stored
        // wall-clock reading describe the same moment.
        self.sync();
        let snapshot = Saved {
            version: pet_core::pet::SAVE_VERSION,
            pet: self.pet.clone(),
            wall_s: self.clock.last_wall_s(),
            best_age_s: self.best_age_s,
        };
        match self.store.save(STATE_KEY, &snapshot) {
            Ok(()) => {
                self.dirty = false;
                self.user_dirty = false;
            }
            Err(e) => {
                log::error!("pet: save failed: {:?}", e.kind());
                self.status = "Save failed".into();
            }
        }
    }

    pub fn on_tick(&mut self) {
        self.sync();
        self.draw();
    }

    pub fn on_foreground(&mut self) {
        self.sync();
        self.screen.refresh_size();
        self.draw();
    }

    pub fn on_background(&mut self) { self.save_if_dirty(); }

    fn act(&mut self, i: usize) {
        self.sync();
        let Some(p) = self.pet.as_mut() else { return };
        let msg = match i {
            0 => p.feed(),
            1 => p.play(),
            2 => p.sleep(),
            3 => p.medicine(),
            4 => p.clean(),
            _ => p.discipline(),
        };
        self.status = msg.to_string();
        self.dirty = true;
        self.user_dirty = true;
        self.save_if_dirty();
    }

    pub fn on_key(&mut self, key: Key) -> Action {
        self.status.clear();
        match self.page {
            Page::Naming => match key {
                Key::Enter => {
                    let p = Pet::new(self.name.as_str());
                    self.pet = Some(p);
                    self.name.clear();
                    self.page = Page::Status;
                    self.dirty = true;
                    self.user_dirty = true;
                    // A new egg starts counting from now.
                    self.clock = Timekeeper::default();
                    self.sync();
                    self.save_if_dirty();
                }
                Key::Backspace => {
                    // Backspace edits the name; on an empty name it leaves.
                    if !self.name.backspace() {
                        return Action::Leave;
                    }
                }
                Key::F(4) => return Action::Leave,
                Key::Left => self.name.left(),
                Key::Right => self.name.right(),
                Key::Char(c) => {
                    self.name.insert(c);
                }
                _ => {}
            },
            Page::Status => {
                let dead = self.pet.as_ref().is_some_and(|p| !p.alive);
                match key {
                    k if k.is_back() => {
                        self.save_if_dirty();
                        return Action::Leave;
                    }
                    Key::Enter if dead => {
                        self.page = Page::Naming;
                    }
                    Key::Enter | Key::Right | Key::Down if !dead => {
                        self.list = ListState::default();
                        self.page = Page::Actions;
                    }
                    Key::Char(c @ '1'..='6') if !dead => self.act((c as u8 - b'1') as usize),
                    _ => {}
                }
            }
            Page::Actions => match key {
                k if k.is_back() => self.page = Page::Status,
                Key::Left => self.page = Page::Status,
                Key::Up => self.list.step(-1, ACTIONS.len(), ACTIONS.len()),
                Key::Down => self.list.step(1, ACTIONS.len(), ACTIONS.len()),
                Key::Char(c @ '1'..='6') => {
                    self.act((c as u8 - b'1') as usize);
                    self.page = Page::Status;
                }
                Key::Enter => {
                    self.act(self.list.selected);
                    self.page = Page::Status;
                }
                _ => {}
            },
        }
        Action::Stay
    }

    // ---- drawing ----

    pub fn draw(&mut self) {
        self.screen.clear();
        match self.page {
            Page::Naming => self.draw_naming(),
            Page::Status => self.draw_status(),
            Page::Actions => self.draw_actions(),
        }
        self.screen.flush();
    }

    fn draw_naming(&self) {
        let s = &self.screen;
        let top = s.header("Pet");
        s.footer("type a name  ⏎ hatch  ⌫ delete/leave");
        let mut y = top + 8;
        y += s.text(y, GlyphStyle::Regular, "Name your pet:");
        y += s.text(y, GlyphStyle::Large, &format!("{}_", self.name.as_str()));
        let shown = self.name.as_str();
        s.text(y + 8, GlyphStyle::Small, &format!("{}/{} letters. Empty = Pixel.", shown.chars().count(), NAME_MAX));
        if self.best_age_s > 0 {
            s.text(y + 30, GlyphStyle::Small, &format!("Longest life so far: {}", fmt_duration(self.best_age_s)));
        }
        if !self.status.is_empty() {
            s.text(y + 50, GlyphStyle::Small, &self.status);
        }
    }

    fn bar(&self, y: isize, label: &str, value: u8) -> isize {
        let s = &self.screen;
        let h = s.line_height(GlyphStyle::Small);
        s.text_in(Point::new(MARGIN, y), Point::new(MARGIN + 66, y + h), GlyphStyle::Small, label);
        let x0 = MARGIN + 70;
        let x1 = s.size().x - 40;
        s.frame(Point::new(x0, y + 2), Point::new(x1, y + h - 3), 1);
        let fill = ((x1 - x0 - 4) as i64 * value.min(100) as i64 / 100) as isize;
        if fill > 0 {
            s.fill_rects([(Point::new(x0 + 2, y + 4), Point::new(x0 + 2 + fill, y + h - 5))], PixelColor::Dark);
        }
        s.text_in(Point::new(x1 + 4, y), Point::new(s.size().x - 2, y + h), GlyphStyle::Small, &value.to_string());
        h
    }

    fn draw_status(&self) {
        let s = &self.screen;
        let Some(p) = self.pet.as_ref() else { return };
        let top = s.header(&format!("{} - {}", p.name, p.stage().name()));
        let hint = if p.alive { "⏎ care  1-6 quick  ⌫ leave" } else { "⏎ new egg  ⌫ leave" };
        let bottom = s.footer(hint);
        let w = s.size().x;
        let mut y = top;
        let lh = s.line_height(GlyphStyle::Monospace);
        for line in p.sprite() {
            s.text_centered(Point::new(0, y), Point::new(w, y + lh), GlyphStyle::Monospace, line);
            y += lh;
        }
        y += 4;
        if !p.alive {
            let how = match p.cause {
                Some(pet_core::Cause::OldAge) => "died of old age",
                _ => "died of neglect",
            };
            y = s.para(y, bottom, GlyphStyle::Regular, &format!("{} {} after {}.", p.name, how, p.age_label()));
            y = s.para(y, bottom, GlyphStyle::Small, &format!("Longest life: {}", fmt_duration(self.best_age_s)));
        } else if p.stage() == Stage::Egg {
            y = s.para(y, bottom, GlyphStyle::Regular, "Something is stirring in the egg...");
        } else {
            y += s.text(y, GlyphStyle::Regular, &format!("{}  {}", p.mood().face(), p.mood().name()));
            y += self.bar(y, "Fed", 100 - p.hunger);
            y += self.bar(y, "Happy", p.happiness);
            y += self.bar(y, "Energy", p.energy);
            y += self.bar(y, "Health", p.health);
            y += s.text(y, GlyphStyle::Small, &format!("Age {}  Wt {}  Disc {}  Poop {}", p.age_label(), p.weight, p.discipline, p.poop));
        }
        if !self.status.is_empty() && y < bottom {
            s.text(y + 4, GlyphStyle::Small, &self.status);
        }
        if self.pet_clock_note() && y + 24 < bottom {
            s.text(bottom - s.line_height(GlyphStyle::Small), GlyphStyle::Small, "Clock not set: pet ages only while on");
        }
    }

    /// True when the wall clock isn't set, so time away from the app (suspend,
    /// reboot) can't be counted.
    fn pet_clock_note(&self) -> bool { self.clock.last_wall_s().is_none() }

    fn draw_actions(&mut self) {
        let s = &self.screen;
        let top = s.header("Care");
        let bottom = s.footer("↑↓ pick  ⏎ do  ⌫ back");
        let items: Vec<String> = ACTIONS.iter().enumerate().map(|(i, a)| format!("{} {}", i + 1, a)).collect();
        s.list(top, bottom, &items, &self.list);
    }
}
