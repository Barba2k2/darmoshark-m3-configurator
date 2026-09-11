# Napkin

## Receiver transport
- 2026-09-11: Status 2 ("no link") is the mouse asleep. Do: move the mouse
  before a hardware run; never tell the user to replug first.
- 2026-09-11: Receiver failures come and go with conditions (sleep, load); a
  passing run on old code proves nothing. Do: capture raw 0x54 frames with a
  probe before and after a transport change.
- 2026-09-11: A receiver write is acked when queued; the mouse applies it up
  to ~200 ms later. Do: read the setting back until it shows (at most ten
  reads, never an error), as `settle_on_receiver` does.
- 2026-09-11: Neither "ready" nor the echo proves a reply is fresh. Do: if the
  buffer already echoes the command, send a primer with a different echo
  first, then poll until the real echo shows.
- 2026-09-11: Negative results read before the transport fix (e.g. "rate does
  not move the nibble") were stale reads. Do: re-test on hardware before
  trusting any conclusion older than the fix.
- 2026-09-11: A polling rate is measurable: open the receiver's mouse
  interface (usage 1/2) shared (`hidapi` feature `macos-shared-device`) and
  time input reports while the mouse moves; p10 of the gaps is the interval.
- 2026-09-11: Same-value write + read-back proves nothing. Do: change the
  setting, read, restore, then assert.

## Vendor bundle
- 2026-09-11: `research/bundle/main.beautified.js` holds one contract class per
  transport (`at.inject(K => "M" === K ...)` is the receiver, `"dms"` the
  cable). Do: read the receiver's class before assuming a cable command works
  there -- the "M" class never sends sleep.

## Port
- 2026-09-11: Comparing two device commands through `diff <(a) <(b)` runs
  them at once and the second hits macOS exclusive access. Do: run them in
  sequence into files, then diff.
- 2026-09-11: Frame parity is checked against `tests/oracle/fixtures/
  python_oracle.json`. The Python and its recorder are gone (removed after
  3dea1a0), so the fixture is frozen. Do: edit it by hand only with hardware
  evidence, and say so in the commit.
