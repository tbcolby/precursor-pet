"""Renode capture sequence for the pet (not yet run).

Run from xous-dev-toolkit:
    python3 scripts/renode_capture.py --app pet --script ../precursor-pet/capture.py

Contract: run(ctl, outdir) -> number of failed steps. The app is already launched.
"""
import os


def run(ctl, outdir) -> int:
    failures = 0

    def shot(name):
        nonlocal failures
        if not ctl.try_screenshot(os.path.join(outdir, name)):
            failures += 1

    shot("01_naming.png")
    ctl.inject_string("Tama")
    ctl.inject_line("")                    # Enter: the egg appears
    ctl.sleep(1.0)
    shot("02_egg.png")
    ctl.timed_key("Right", after=1.0)      # care menu
    shot("03_care.png")
    ctl.timed_key("BackSpace", after=1.0)  # back to the pet
    ctl.timed_key("F4", after=1.0)         # leave the app
    return failures
