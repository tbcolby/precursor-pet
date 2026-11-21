use pet_core::pet::{ADULT_S, CHILD_S, HATCH_S, SAVE_VERSION, SENIOR_S, TEEN_S};
use pet_core::*;

const MIN: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 86_400;

fn baby() -> Pet {
    let mut p = Pet::new("Tama");
    p.advance(HATCH_S, false);
    assert_eq!(p.stage(), Stage::Baby);
    p
}

#[test]
fn egg_has_no_needs_and_hatches_after_five_minutes() {
    let mut p = Pet::new("Tama");
    let (h, hap) = (p.hunger, p.happiness);
    assert!(p.advance(HATCH_S - 1, false).is_empty());
    assert_eq!((p.stage(), p.hunger, p.happiness), (Stage::Egg, h, hap));
    for act in [p.feed(), p.play(), p.sleep(), p.medicine(), p.clean(), p.discipline()] {
        assert_eq!(act, "...");
    }
    let ev = p.advance(1 + MIN, false);
    assert_eq!(p.stage(), Stage::Baby);
    assert_eq!(ev, vec!["Tama hatched!".to_string()]);
}

#[test]
fn stages_follow_real_age() {
    let mut p = Pet::new("Tama");
    for (age, stage) in [
        (HATCH_S, Stage::Baby),
        (CHILD_S, Stage::Child),
        (TEEN_S, Stage::Teen),
        (ADULT_S, Stage::Adult),
        (SENIOR_S, Stage::Senior),
    ] {
        assert_eq!(Stage::of_age(age - 1), match stage {
            Stage::Baby => Stage::Egg,
            Stage::Child => Stage::Baby,
            Stage::Teen => Stage::Child,
            Stage::Adult => Stage::Teen,
            _ => Stage::Adult,
        });
        assert_eq!(Stage::of_age(age), stage);
    }
    // fed and cared for at every step the pet reaches each stage
    let mut seen = vec![p.stage()];
    while p.age_s < SENIOR_S {
        p.advance(30 * MIN, false);
        p.hunger = 0;
        p.happiness = 100;
        p.poop = 0;
        p.energy = 100;
        p.sick = false;
        if seen.last() != Some(&p.stage()) {
            seen.push(p.stage());
        }
    }
    assert_eq!(seen, vec![Stage::Egg, Stage::Baby, Stage::Child, Stage::Teen, Stage::Adult, Stage::Senior]);
}

#[test]
fn unattended_pet_takes_hours_to_starve_not_minutes_regression() {
    // Audit: the pet starved about 5 minutes after its last meal and lived 15 minutes in all.
    let mut p = baby();
    let start = p.age_s;
    let mut died_after = None;
    for m in 1..=(48 * 60) {
        p.advance(MIN, false);
        if !p.alive {
            died_after = Some(m);
            break;
        }
    }
    let m = died_after.expect("a pet left alone for two days must die");
    assert!(m > 8 * 60 && m < 20 * 60, "died after {m} minutes");
    assert_eq!(p.cause, Some(Cause::Neglect));
    assert_eq!(p.health, 0);
    assert!(p.age_s - start <= 20 * HOUR);
}

#[test]
fn one_hour_of_needs() {
    let mut p = baby();
    let (h, hap, e) = (p.hunger, p.happiness, p.energy);
    p.advance(HOUR, false);
    // minutes 6..=65 since birth (t counts from creation: 5 min egg + 60)
    assert!(p.hunger >= h + 8 && p.hunger <= h + 9, "hunger {}", p.hunger);
    assert!(p.happiness < hap && p.happiness >= hap - 8);
    assert!(p.energy < e && p.energy >= e - 7);
    assert!(p.alive && p.health == 100);
}

#[test]
fn elapsed_time_can_be_chunked_any_way() {
    let mut a = Pet::new("Tama");
    let mut b = Pet::new("Tama");
    let total = 3 * HOUR + 17 * MIN + 41;
    a.advance(total, false);
    let mut left = total;
    for chunk in [1u64, 59, 60, 7, 3600, 13, 500, 20, 20, 20] {
        let c = chunk.min(left);
        b.advance(c, false);
        left -= c;
    }
    b.advance(left, false);
    assert_eq!(a, b);
    // sub-minute remainders accumulate instead of being dropped
    let mut c = Pet::new("Tama");
    for _ in 0..300 {
        c.advance(1, false);
    }
    assert_eq!(c.stage(), Stage::Baby);
}

#[test]
fn catching_up_equals_living_through_it() {
    // Foreground catch-up over a pocketed evening gives the same pet as ticking along.
    let mut live = baby();
    let mut caught_up = live.clone();
    for _ in 0..(3 * HOUR / 20) {
        live.advance_auto(20);
    }
    caught_up.advance_auto(3 * HOUR);
    assert_eq!(live, caught_up);
}

#[test]
fn nothing_ticks_without_elapsed_time() {
    let mut p = baby();
    let before = p.clone();
    assert!(p.advance(0, false).is_empty());
    assert_eq!(p, before);
}

#[test]
fn a_pet_cannot_die_while_away() {
    let mut p = baby();
    let ev = p.advance_auto(7 * DAY);
    assert!(p.alive);
    assert!(p.health >= 1);
    assert!(p.sick, "neglect still has consequences");
    assert_eq!(p.hunger, 100);
    assert!(ev.iter().any(|e| e.contains("sick")));
    // But if it is then left on screen unfed it can.
    p.advance(2 * HOUR, false);
    assert!(!p.alive);
}

#[test]
fn old_age_ends_every_life_even_while_away() {
    let mut p = baby();
    p.advance(14 * DAY, true);
    assert!(p.alive);
    let ev = p.advance(14 * DAY, true);
    assert!(!p.alive);
    assert_eq!(p.cause, Some(Cause::OldAge));
    assert!(ev.iter().any(|e| e.contains("old age")));
    // age stops at death
    let age = p.age_s;
    p.advance(DAY, false);
    assert_eq!(p.age_s, age);
    assert!(age <= LIFESPAN_S + TICK_S);
}

#[test]
fn huge_elapsed_time_is_capped() {
    let mut p = Pet::new("Tama");
    p.advance(u64::MAX, true);
    assert_eq!(p.age_s, MAX_CATCHUP_S);
    let mut q = Pet::new("Tama");
    q.advance(MAX_CATCHUP_S, true);
    assert_eq!(p, q);
    // carry near the maximum doesn't overflow either
    let mut r = Pet::new("Tama");
    r.advance(59, false);
    r.advance(u64::MAX, true);
}

#[test]
fn dead_pet_stays_dead_and_actions_refuse() {
    let mut p = baby();
    p.advance(2 * DAY, false);
    assert!(!p.alive);
    let snapshot = p.clone();
    assert!(p.advance(DAY, false).is_empty());
    for act in [p.feed(), p.play(), p.sleep(), p.medicine(), p.clean(), p.discipline()] {
        assert_eq!(act, "...", "clean on a dead pet used to work");
    }
    assert_eq!(p, snapshot);
}

#[test]
fn feeding_and_playing() {
    let mut p = baby();
    p.hunger = 5;
    assert_eq!(p.feed(), "Not hungry!");
    p.hunger = 50;
    assert_eq!(p.feed(), "Yum!");
    assert_eq!(p.hunger, 20);
    p.hunger = 99;
    p.weight = 99;
    p.feed();
    assert_eq!((p.hunger, p.weight), (69, 99));
    p.energy = 9;
    assert_eq!(p.play(), "Too tired!");
    p.energy = 50;
    p.happiness = 90;
    p.hunger = 98;
    p.weight = 1;
    assert_eq!(p.play(), "Wheee!");
    assert_eq!((p.happiness, p.energy, p.hunger, p.weight), (100, 35, 100, 1));
}

#[test]
fn sleeping_restores_energy_and_wakes_by_itself() {
    let mut p = baby();
    assert_eq!(p.sleep(), "Not sleepy.");
    p.energy = 40;
    assert_eq!(p.sleep(), "Zzz...");
    assert!(p.sleeping);
    assert_eq!(p.mood(), Mood::Asleep);
    assert_eq!(p.feed(), "Zzz... (wake it first)");
    assert_eq!(p.play(), "Zzz... (wake it first)");
    let h = p.hunger;
    let ev = p.advance(40 * MIN, false);
    assert!(!p.sleeping);
    assert!(p.energy >= 99, "energy {}", p.energy);
    assert!(ev.iter().any(|e| e.contains("woke up")));
    assert!(p.hunger <= h + 4, "hunger grows slowly while asleep");
    // waking manually
    p.energy = 50;
    p.sleep();
    assert_eq!(p.sleep(), "Awake.");
    assert!(!p.sleeping);
}

#[test]
fn sickness_and_medicine() {
    let mut p = baby();
    assert_eq!(p.medicine(), "Not sick!");
    p.health = 35;
    p.advance(MIN, false);
    assert!(p.sick);
    assert_eq!(p.mood(), Mood::Sick);
    let hap = p.happiness;
    assert_eq!(p.medicine(), "Medicine given.");
    assert!(!p.sick);
    assert!(p.health >= 60);
    assert_eq!(p.happiness, hap - 10);
}

#[test]
fn cleaning() {
    let mut p = baby();
    assert_eq!(p.clean(), "Already clean!");
    p.advance(4 * HOUR, false);
    assert!(p.poop >= 1);
    assert_eq!(p.clean(), "Cleaned up!");
    assert_eq!(p.poop, 0);
}

#[test]
fn weight_and_discipline_matter() {
    // Heavy pets lose health; the audit found weight did nothing.
    let mut fit = baby();
    let mut heavy = baby();
    heavy.weight = 90;
    for p in [&mut fit, &mut heavy] {
        for _ in 0..120 {
            p.advance(MIN, false);
            p.hunger = 0;
            p.happiness = 100;
            p.energy = 100;
        }
    }
    assert_eq!(fit.health, 100);
    assert!(heavy.health < 100, "health {}", heavy.health);
    // Discipline: an unruly pet gets bored faster, and discipline fades slowly.
    let mut wild = baby();
    wild.discipline = 10;
    let mut tidy = baby();
    tidy.discipline = 90;
    wild.advance(HOUR, false);
    tidy.advance(HOUR, false);
    assert!(wild.happiness < tidy.happiness);
    assert_eq!(tidy.discipline, 89);
    let d = tidy.discipline;
    tidy.discipline();
    assert_eq!(tidy.discipline, (d + 10).min(100));
    let mut t2 = baby();
    t2.discipline = 99;
    t2.discipline();
    assert_eq!(t2.discipline, 100);
}

#[test]
fn moods() {
    let mut p = baby();
    p.happiness = 90;
    p.hunger = 10;
    assert_eq!(p.mood(), Mood::Happy);
    p.hunger = 60;
    assert_eq!(p.mood(), Mood::Content);
    p.happiness = 40;
    assert_eq!(p.mood(), Mood::Bored);
    p.happiness = 10;
    assert_eq!(p.mood(), Mood::Sad);
    assert!(!p.mood().face().is_empty());
}

#[test]
fn stats_stay_in_range_over_a_lifetime() {
    let mut p = baby();
    for i in 0..(21 * 24 * 6) {
        p.advance(10 * MIN, i % 40 == 0);
        match i % 7 {
            0 => {
                p.feed();
            }
            1 => {
                p.play();
            }
            2 => {
                p.clean();
            }
            3 => {
                p.sleep();
            }
            4 => {
                p.medicine();
            }
            5 => {
                p.discipline();
            }
            _ => {}
        }
        for v in [p.hunger, p.happiness, p.energy, p.health, p.discipline] {
            assert!(v <= 100);
        }
        assert!(p.poop <= 5 && p.weight >= 1 && p.weight <= 99);
    }
}

#[test]
fn names_are_cleaned() {
    assert_eq!(Pet::new("").name, "Pixel");
    assert_eq!(Pet::new("   ").name, "Pixel");
    assert_eq!(Pet::new("  Bo\nb ").name, "Bob");
    assert_eq!(Pet::new("12345678901234567890").name.chars().count(), NAME_MAX);
    assert_eq!(Pet::new("Zoë🐙").name, "Zoë🐙");
}

#[test]
fn sprites_exist_for_every_stage() {
    let mut p = Pet::new("Tama");
    for age in [0, HATCH_S, CHILD_S, TEEN_S, ADULT_S, SENIOR_S] {
        p.age_s = age;
        assert!(p.sprite().len() >= 4);
    }
    assert_eq!(baby().age_label(), "5m");
    p.age_s = 2 * DAY + 3 * HOUR;
    assert_eq!(p.age_label(), "2d 3h");
    p.age_s = 90 * MIN;
    assert_eq!(p.age_label(), "1h 30m");
}

#[test]
fn save_round_trip() {
    let mut p = baby();
    p.advance(5 * HOUR + 17, false);
    let s = Saved { version: SAVE_VERSION, pet: Some(p.clone()), wall_s: Some(123_456), best_age_s: 99 };
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.len() < 600, "{}", json.len());
    let back: Saved = serde_json::from_str(&json).unwrap();
    assert_eq!(back, s);
    assert!(back.check().is_ok());
    // identical future
    let mut q = back.pet.unwrap();
    p.advance(3 * HOUR, false);
    q.advance(3 * HOUR, false);
    assert_eq!(p, q);
    assert!(Saved::default().check().is_ok());
}

#[test]
fn save_validation() {
    let p = baby();
    let good = Saved { version: SAVE_VERSION, pet: Some(p), wall_s: None, best_age_s: 0 };
    let edit = |f: &dyn Fn(&mut serde_json::Value)| -> Result<(), LoadError> {
        let mut v = serde_json::to_value(&good).unwrap();
        f(&mut v);
        let s: Saved = serde_json::from_value(v).unwrap();
        s.check()
    };
    assert_eq!(edit(&|_| {}), Ok(()));
    assert_eq!(edit(&|v| v["version"] = 9.into()), Err(LoadError::NewerVersion(9)));
    for (field, val) in [
        ("hunger", 101),
        ("happiness", 101),
        ("energy", 200),
        ("health", 0),
        ("weight", 0),
        ("weight", 100),
        ("poop", 6),
        ("carry_s", 60),
        ("age_s", 99 * 86_400),
    ] {
        assert_eq!(edit(&|v| v["pet"][field] = val.into()), Err(LoadError::Corrupt), "{field}={val}");
    }
    assert_eq!(edit(&|v| v["pet"]["name"] = "".into()), Err(LoadError::Corrupt));
    assert_eq!(edit(&|v| v["pet"]["name"] = "x".repeat(40).into()), Err(LoadError::Corrupt));
    assert_eq!(edit(&|v| v["pet"]["name"] = "a\u{7}".into()), Err(LoadError::Corrupt));
    assert_eq!(edit(&|v| v["pet"]["alive"] = false.into()), Err(LoadError::Corrupt)); // dead without a cause
}

#[test]
fn full_boot_cycle_with_the_timekeeper() {
    // Evening: foreground, saved at wall 10_000 s.
    let mut tk = Timekeeper::default();
    let mut p = baby();
    tk.elapsed_secs(Some(10_000), 1_000);
    let saved = Saved { version: SAVE_VERSION, pet: Some(p.clone()), wall_s: tk.last_wall_s(), best_age_s: 0 };

    // Reboot 5 hours later; the ticktimer restarts near zero.
    let mut tk2 = Timekeeper::from_saved(saved.wall_s);
    let mut q = saved.pet.unwrap();
    let e = tk2.elapsed_secs(Some(10_000 + 5 * HOUR), 2_000);
    assert_eq!(e, 5 * HOUR);
    q.advance_auto(e);
    p.advance_auto(5 * HOUR);
    assert_eq!(p, q);
    assert!(q.hunger > 60 && q.alive);
}

/// Regression: syncing more often than once a second with the wall clock
/// unset used to lose each call's sub-second remainder, so the pet aged slowly.
#[test]
fn frequent_syncs_without_wall_clock_keep_full_time() {
    let mut tk = Timekeeper::default();
    let total: u64 = (0..=120u64).map(|i| tk.elapsed_secs(None, i * 500)).sum();
    assert_eq!(total, 60);
}
