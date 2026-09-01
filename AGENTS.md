# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project

Open-source configurator for the Darmoshark M3 (Attack Shark M3) mouse on macOS.
The `dms` wire protocol was reverse engineered from the vendor's WebHID bundle
(`darmoshark.cc`) and validated against real hardware; `PROTOCOL.md` is the
authoritative spec and must be updated alongside any protocol change.

Dependencies: `hidapi`, `PySide6-Essentials`. Everything runs from `.venv` with
`PYTHONPATH=src` — there is no package install step.

## Commands

```bash
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt
```

```bash
PYTHONPATH=src .venv/bin/python -m unittest discover -s tests
```

Single test:

```bash
PYTHONPATH=src .venv/bin/python -m unittest tests.test_packets.DpiPacketTest.test_encodes_five_levels_little_endian
```

GUI: `PYTHONPATH=src .venv/bin/python src/gui/app.py`
CLI: `PYTHONPATH=src .venv/bin/python src/cli.py <command>` (`list`, `capabilities`,
`colors`, `battery`, `dfu`, `dpi`, `use`, `rate`, `debounce`, `lod`, `sleep`,
`profile`, `button`, `reset`, `info`, `buttons`, `bond`).

No linter or type checker is configured.

## Architecture

Three layers, strictly one class per file.

**`src/darmoshark/`** — protocol and transport.
- `protocol.py` (`DarmosharkProtocol`) holds every report id, opcode, usage page
  and limit; `dms_commands.py` (`DmsCommands`) holds the 31-opcode table and the
  short/long report routing. Constants live only in these two files.
- `*_packet.py` are pure builders: static methods returning `(reportId, payload)`
  bytes, validating ranges before anything reaches the hardware. They never touch
  a device, which is why the tests can assert byte-identical frames.
- `*_info.py` are pure decoders for replies (`BaseInfo`, `CableInfo`, `DfuInfo`,
  `DongleBaseInfo`).
- `device.py` (`DarmosharkDevice`) is the only file that talks to `hid`.
- `mouse_configurator.py` (`MouseConfigurator`) composes builders + device into
  the high-level operations; both CLI and GUI go through it, never around it.
- `device_profile.py` reads the vendor's `m3_profile.json` for offline
  capabilities (DPI range, LED colours per level, polling rates, lift-off steps).

**`src/gui/`** — PySide6. `main_window.py` wires widgets to `mouse_service.py`,
which opens a fresh HID handle per operation (so unplugging the mouse cannot
wedge the window). Widgets in `gui/widgets/` are string-free and take text as
arguments; all copy lives in `labels.py` (currently Portuguese) and all colours
and metrics in `theme.py`.

**`src/cli.py`** — argparse front-end over the same `MouseConfigurator`.

## Two transports, one payload

The critical asymmetry to keep in mind when touching `device.py` or
`mouse_configurator.py`:

Both live on usage page `0x8C` and are distinguished by product id: the mouse
itself is `0xFF12`, the receiver `0xFF30` (`DarmosharkProtocol.dongleProductIds`,
read through `DarmosharkDevice.usesDongleTransport`). The receiver has no vendor
usage page of its own, and the reports `0xB3`/`0xB5` that the `dms` contract
talks about exist on neither interface — writing them reaches nothing.

| | Cable (feature `0x52`, 64 B) | Receiver (feature `0x51`, 20 B) |
|---|---|---|
| Writes | fire-and-forget, silent | fire-and-forget, `0xE4` on input `0x54` |
| Reads | always the identity block, never config | full snapshot, opcode `0x07` |
| Long commands | same `0x52` | feature `0x52`, 64 B (button reads) |
| DPI levels | max 5 (`0x40`; extended `0x44` ignored) | max 5, no route for `0x44` |
| Bootloader | the mouse's | the **receiver's** own |

Reading over the receiver has two traps, both encoded in `requestDongle`:

- The `0xE4` status is `0` or `4` while pending, `1` when the reply is ready in
  the feature report, and **`2` when the RF link is dead**. A dead link never
  heals — every later command answers `2` while the cursor keeps moving, because
  HID input rides a separate path. Only replugging the receiver recovers it.
- The feature buffer holds the *previous* reply until the new one lands, so a
  read that arrives early looks like a valid answer to the wrong question. The
  echoed opcode is checked on every reply; commands that address a slot pass
  `echoBytes=2` so the index is checked too (all five buttons share one opcode).

Still unverified after the receiver work: `setReportRate`. Neither
`ReportRatePacket`'s form (one index byte per level) nor the bundle's `M`-contract
form (uint16 Hz per level) moves the rate nibble of the snapshot, and the
nibble's own mapping is unconfirmed. Do not "fix" the builder to the other
layout without hardware evidence — that swaps one unverified guess for another.

`DarmosharkDevice.usesCableTransport` and `usesDongleTransport` select the path;
`MouseConfigurator._sendAcknowledged` skips ACK checking on both, since neither
acknowledges a write. A silent device means a *successful* write, not a failure
— do not add error handling that treats it as one.

## Constraints

- Never widen the DPI range or level count past `DarmosharkProtocol` limits — a
  malformed packet can leave the mouse in an inconsistent onboard profile.
- Reads that need the receiver must fail with a message saying so, not silently
  return the identity block.
- `research/` is gitignored vendor bundle scratch; `reference/` holds the public
  vendor JSON definitions and is tracked.
- Docs are bilingual: any change to `README.md`/`PROTOCOL.md` must be mirrored in
  the `.pt-BR.md` counterpart.
