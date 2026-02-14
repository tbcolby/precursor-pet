//! State machine for Tamagotchi.
//! States: Naming, Status, Action

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use crate::pet::Pet;

const KEY_UP: char = '\u{F700}';
const KEY_DOWN: char = '\u{F701}';
const KEY_LEFT: char = '\u{F702}';
const KEY_RIGHT: char = '\u{F703}';
const KEY_ENTER: char = '\u{000D}';
const KEY_BACKSPACE: char = '\u{0008}';
const KEY_MENU: char = '\u{2234}';

pub const ACTIONS: &[&str] = &["Feed", "Play", "Sleep", "Medicine", "Clean", "Discipline"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState { Naming, Status, Action }

pub struct PetApp {
    pub state: AppState,
    pub needs_redraw: bool,
    pub pet: Option<Pet>,
    pub name_input: String,
    pub action_cursor: usize,
    pub messages: Vec<String>,
    pub message_timer: u8,
}

impl PetApp {
    pub fn new() -> Self {
        Self {
            state: AppState::Naming,
            needs_redraw: true,
            pet: None,
            name_input: String::new(),
            action_cursor: 0,
            messages: Vec::new(),
            message_timer: 0,
        }
    }

    pub fn tick(&mut self) {
        if let Some(ref mut pet) = self.pet {
            if let Some(msg) = pet.tick() {
                self.messages.push(msg);
                self.message_timer = 10;
            }
            self.needs_redraw = true;
        }
        if self.message_timer > 0 { self.message_timer -= 1; }
    }

    pub fn latest_message(&self) -> Option<&String> {
        if self.message_timer > 0 { self.messages.last() } else { None }
    }

    pub fn handle_key(&mut self, key: char) -> bool {
        self.needs_redraw = true;
        match self.state {
            AppState::Naming => self.key_naming(key),
            AppState::Status => self.key_status(key),
            AppState::Action => self.key_action(key),
        }
    }

    fn key_naming(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => return false,
            KEY_ENTER => {
                if self.name_input.is_empty() { self.name_input = String::from("Pixel"); }
                self.pet = Some(Pet::new(&self.name_input));
                self.state = AppState::Status;
            }
            KEY_BACKSPACE => { self.name_input.pop(); }
            c if c >= ' ' && c <= '~' => {
                if self.name_input.len() < 16 { self.name_input.push(c); }
            }
            _ => {}
        }
        true
    }

    fn key_status(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => return false,
            KEY_ENTER | KEY_RIGHT => { self.action_cursor = 0; self.state = AppState::Action; }
            _ => {}
        }
        true
    }

    fn key_action(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT => { self.state = AppState::Status; }
            KEY_UP => { if self.action_cursor > 0 { self.action_cursor -= 1; } }
            KEY_DOWN => { if self.action_cursor < ACTIONS.len() - 1 { self.action_cursor += 1; } }
            KEY_ENTER => {
                if let Some(ref mut pet) = self.pet {
                    let msg = match self.action_cursor {
                        0 => pet.feed(),
                        1 => pet.play(),
                        2 => pet.sleep(),
                        3 => pet.medicine(),
                        4 => pet.clean(),
                        5 => pet.discipline(),
                        _ => "...",
                    };
                    self.messages.push(String::from(msg));
                    self.message_timer = 8;
                    self.state = AppState::Status;
                }
            }
            _ => {}
        }
        true
    }
}
