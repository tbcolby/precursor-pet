#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

//! Virtual pet for Precursor. The time model, needs and save format live in
//! `pet-core` (host-tested); this crate is the Xous shell.
//!
//! The pet is not ticked by a timer: every sync asks the clocks how much real
//! time has passed and applies it in one go, so the pet ages through
//! backgrounding, suspend and reboot. The pump only refreshes the screen
//! while the app is in front.

mod app;

use num_traits::{FromPrimitive, ToPrimitive};
use precursor_appkit::{app as kit, keys, pump, store};

/// xous-names server name. Any unique string; NOT the GAM app name.
const SERVER_NAME: &str = "_Pet_";

/// Redraw/sync interval while in front. The pet lives in one-minute steps.
const REFRESH_MS: usize = 30_000;

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub(crate) enum AppOp {
    Redraw = 0,
    Rawkeys,
    FocusChange,
    Tick,
    BasisChanged,
    Quit,
}

fn main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("pet PID is {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns.register_name(SERVER_NAME, None).expect("can't register server");

    // GAM app name comes from apps/manifest.json key "pet".
    let reg = kit::register(
        &xns,
        sid,
        gam::APP_NAME_PET,
        gam::UxType::Framebuffer,
        kit::Opcodes {
            redraw: AppOp::Redraw.to_u32().unwrap(),
            rawkeys: Some(AppOp::Rawkeys.to_u32().unwrap()),
            focus_change: Some(AppOp::FocusChange.to_u32().unwrap()),
            got_input: None,
        },
    );

    let store = store::Store::open("pet");
    store::watch_bases(xous::connect(sid).unwrap(), AppOp::BasisChanged.to_usize().unwrap());
    let pump = pump::Pump::spawn(xous::connect(sid).unwrap(), AppOp::Tick.to_usize().unwrap(), REFRESH_MS);

    let mut app = app::App::new(reg.screen, store, &xns);

    loop {
        let msg = xous::receive_message(sid).unwrap();
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(AppOp::Redraw) => app.draw(),
            Some(AppOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                for key in keys::decode_rawkeys([k1, k2, k3, k4]) {
                    if app.on_key(key) == app::Action::Leave {
                        kit::leave(app.screen());
                    }
                }
                app.draw();
            }),
            Some(AppOp::FocusChange) => xous::msg_scalar_unpack!(msg, code, _, _, _, {
                match kit::focus_state(code) {
                    gam::FocusState::Foreground => {
                        // Catch up on everything that happened while away.
                        app.on_foreground();
                        pump.run();
                    }
                    gam::FocusState::Background => {
                        pump.stop();
                        app.on_background();
                    }
                }
            }),
            Some(AppOp::Tick) => {
                app.on_tick();
                pump::ack(&msg);
            }
            Some(AppOp::BasisChanged) => app.reload(),
            Some(AppOp::Quit) => break,
            None => log::warn!("unknown opcode {:?}", msg.body.id()),
        }
    }
    pump.quit();
    kit::shutdown(&xns, sid)
}
