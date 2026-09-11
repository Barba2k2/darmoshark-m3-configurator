# Napkin

## Receiver transport
- 2026-09-11: A write over the receiver lands 300-800 ms later (one `E4 00`
  on `0x54`). Reading before that returns the old value. Do: make the write
  wait for its frame, then read.
- 2026-09-11: The ack never names the command (`E4 <status> 00`). Do: drain
  `0x54` before sending, open the feature buffer only on status 1 (or no ack at
  all, as the bond read does), resend while pending.
- 2026-09-11: Same-value write + read-back proves nothing. Do: change the
  setting, read, restore, then assert.

## Port
- 2026-09-11: Comparing two device commands through `diff <(a) <(b)` runs
  them at once and the second hits macOS exclusive access. Do: run them in
  sequence into files, then diff.
- 2026-09-11: Frame parity is checked against `tests/oracle/fixtures/
  python_oracle.json`. Do: regenerate it only with `record_python_oracle.py`, never by
  hand, and only while the Python still exists.
