//! UI rendering for Tamagotchi.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use gam::{Gam, Point, Line, TextView, TextBounds, GlyphStyle};
use crate::app::{PetApp, AppState, ACTIONS};

const SCREEN_W: i16 = 336;
const HEADER_H: i16 = 30;
const FOOTER_Y: i16 = 490;

fn draw_header(gam: &Gam, title: &str) {
    gam.draw_rectangle(Point::new(0, 0), Point::new(SCREEN_W, HEADER_H)).ok();
    let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(8, 2), Point::new(SCREEN_W - 8, HEADER_H)));
    tv.style = GlyphStyle::Bold; tv.invert = true;
    tv.text = String::from(title);
    gam.post_textview(&mut tv).ok();
}

fn draw_footer(gam: &Gam, text: &str) {
    gam.draw_line(Line::new(Point::new(0, FOOTER_Y), Point::new(SCREEN_W, FOOTER_Y))).ok();
    let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(8, FOOTER_Y + 4), Point::new(SCREEN_W - 8, FOOTER_Y + 40)));
    tv.style = GlyphStyle::Small;
    tv.text = String::from(text);
    gam.post_textview(&mut tv).ok();
}

fn text_at(gam: &Gam, x: i16, y: i16, h: i16, s: &str, style: GlyphStyle) {
    let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(x, y), Point::new(SCREEN_W - 8, y + h)));
    tv.style = style; tv.text = String::from(s);
    gam.post_textview(&mut tv).ok();
}

fn draw_bar(gam: &Gam, x: i16, y: i16, label: &str, value: u8, width: i16) {
    text_at(gam, x, y, 16, label, GlyphStyle::Small);
    let bar_x = x + 70;
    // Draw bar outline
    gam.draw_line(Line::new(Point::new(bar_x, y + 2), Point::new(bar_x + width, y + 2))).ok();
    gam.draw_line(Line::new(Point::new(bar_x, y + 12), Point::new(bar_x + width, y + 12))).ok();
    gam.draw_line(Line::new(Point::new(bar_x, y + 2), Point::new(bar_x, y + 12))).ok();
    gam.draw_line(Line::new(Point::new(bar_x + width, y + 2), Point::new(bar_x + width, y + 12))).ok();
    // Fill
    let fill_w = ((value as i32) * (width as i32) / 100) as i16;
    if fill_w > 0 {
        gam.draw_rectangle(Point::new(bar_x + 1, y + 3), Point::new(bar_x + fill_w, y + 11)).ok();
    }
    // Value text
    text_at(gam, bar_x + width + 4, y, 16, &format!("{}", value), GlyphStyle::Small);
}

pub fn draw(gam: &Gam, app: &PetApp) {
    gam.clear().ok();
    match app.state {
        AppState::Naming => draw_naming(gam, app),
        AppState::Status => draw_status(gam, app),
        AppState::Action => draw_action(gam, app),
    }
    gam.redraw().ok();
}

fn draw_naming(gam: &Gam, app: &PetApp) {
    draw_header(gam, "Tamagotchi");

    let mut y: i16 = 80;
    text_at(gam, 12, y, 22, "Name your pet:", GlyphStyle::Regular);
    y += 30;

    gam.draw_line(Line::new(Point::new(8, y + 24), Point::new(SCREEN_W - 8, y + 24))).ok();
    let display = if app.name_input.is_empty() { String::from("Pixel|") } else { format!("{}|", app.name_input) };
    text_at(gam, 12, y, 22, &display, GlyphStyle::Regular);
    y += 50;

    // Egg ASCII art
    let egg = &["  ___  ", " /   \\ ", "|  o  |", " \\___/ "];
    for line in egg {
        text_at(gam, 120, y, 22, line, GlyphStyle::Regular);
        y += 22;
    }

    draw_footer(gam, "Enter: hatch  Menu: quit");
}

fn draw_status(gam: &Gam, app: &PetApp) {
    if let Some(ref pet) = app.pet {
        let header = if pet.alive {
            format!("{} ({}) {}", pet.name, pet.stage.name(), pet.mood.face())
        } else {
            format!("{} - R.I.P.", pet.name)
        };
        draw_header(gam, &header);

        let mut y: i16 = 38;

        // ASCII art sprite
        for line in pet.sprite() {
            text_at(gam, 120, y, 22, line, GlyphStyle::Regular);
            y += 22;
        }
        y += 10;

        // Mood and poop
        let mood_line = format!("Mood: {} {}  Age: {}", pet.mood.name(), pet.mood.face(), pet.age);
        text_at(gam, 12, y, 18, &mood_line, GlyphStyle::Small);
        y += 18;

        if pet.poop_count > 0 {
            let poop: String = (0..pet.poop_count).map(|_| '*').collect();
            text_at(gam, 12, y, 18, &format!("Poop: {}", poop), GlyphStyle::Small);
            y += 18;
        }
        y += 6;

        // Stat bars
        let bar_w: i16 = 150;
        draw_bar(gam, 12, y, "Hunger:", 100 - pet.hunger, bar_w); y += 20; // invert: show fullness
        draw_bar(gam, 12, y, "Happy:", pet.happiness, bar_w); y += 20;
        draw_bar(gam, 12, y, "Energy:", pet.energy, bar_w); y += 20;
        draw_bar(gam, 12, y, "Health:", pet.health, bar_w); y += 20;
        draw_bar(gam, 12, y, "Discip:", pet.discipline, bar_w); y += 24;

        text_at(gam, 12, y, 18, &format!("Weight: {}  Stage: {}", pet.weight, pet.stage.name()), GlyphStyle::Small);

        // Message overlay
        if let Some(msg) = app.latest_message() {
            let msg_y: i16 = 420;
            gam.draw_rectangle(Point::new(20, msg_y - 2), Point::new(SCREEN_W - 20, msg_y + 22)).ok();
            let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(24, msg_y), Point::new(SCREEN_W - 24, msg_y + 20)));
            tv.style = GlyphStyle::Regular; tv.invert = true;
            tv.text = msg.clone();
            gam.post_textview(&mut tv).ok();
        }

        draw_footer(gam, "Enter/Right: actions  Menu: quit");
    }
}

fn draw_action(gam: &Gam, app: &PetApp) {
    draw_header(gam, "Actions");

    let start_y: i16 = 50;
    let line_h: i16 = 44;

    for (i, &action) in ACTIONS.iter().enumerate() {
        let y = start_y + (i as i16) * line_h;
        let selected = i == app.action_cursor;

        if selected {
            gam.draw_rectangle(Point::new(4, y - 2), Point::new(SCREEN_W - 4, y + line_h - 8)).ok();
        }

        let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(12, y), Point::new(SCREEN_W - 12, y + 22)));
        tv.style = GlyphStyle::Regular;
        if selected { tv.invert = true; }
        tv.text = String::from(action);
        gam.post_textview(&mut tv).ok();
    }

    draw_footer(gam, "Up/Down: browse  Enter: do  Left: back");
}
