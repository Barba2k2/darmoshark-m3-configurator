# Napkin

## Receiver transport
- 2026-09-11: A write over the receiver lands 300-800 ms later (one `E4 00`
  on `0x54`). Reading before that returns the old value. Do: make the write
  wait for its frame, then read.
- 2026-09-11: The ack never names the command (`E4 <status> 00`). Do: drain
  `0x54` before sending, open the feature buffer only on status 1 (or no ack at
  all, as the bond read does), resend while pending.
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
  python_oracle.json`. Do: regenerate it only with `record_python_oracle.py`, never by
  hand, and only while the Python still exists.
