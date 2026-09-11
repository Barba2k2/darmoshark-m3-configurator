# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Open-source configurator for the Darmoshark M3 (Attack Shark M3) mouse on macOS.
The `dms` wire protocol was reverse engineered from the vendor's WebHID bundle
(`darmoshark.cc`) and validated against real hardware; `PROTOCOL.md` is the
authoritative spec and must be updated alongside any protocol change.

Rust workspace (`crates/darmoshark`, `crates/cli`, `app/src-tauri`) plus a
React webview (`app/src`, pnpm). The original Python implementation was
removed after the port; what it produced survives only as frozen parity
fixtures.

## Commands

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
```

```bash
cd app && pnpm install && pnpm typecheck && pnpm lint && pnpm test
```

Single test: `cargo test -p darmoshark --test packets dpi_packet_test::encodes_five_levels_little_endian`

Hardware (skips itself without the variable; each write changes a setting,
reads it back, then restores it):

```bash
DARMOSHARK_HARDWARE=1 cargo test --test hardware -- --test-threads=1 --nocapture
```

GUI: `cd app && pnpm tauri dev` (compiles the Rust side; ask before running it).
CLI: `cargo run -q -p darmoshark-cli -- <command>` (binary `dms`; `list`,
`capabilities`, `colors`, `battery`, `dfu`, `dpi`, `use`, `rate`, `debounce`,
`lod`, `sleep`, `profile`, `button`, `reset`, `info`, `buttons`, `bond`).

## Architecture

One type per file, 2-space indent (`rustfmt.toml`, `.editorconfig`), constants
in camelCase under `#![allow(non_upper_case_globals)]`.

**`crates/darmoshark`** — protocol and transport, in layer folders.
- `protocol/`: `DarmosharkProtocol` holds every report id, opcode, usage page
  and limit; `DmsCommands` the 31-opcode table and the short/long routing.
  Constants live only there.
- `packets/`: pure builders returning a `Packet` (report id + payload),
  validating ranges before anything reaches the hardware. They never touch a
  device, which is why tests can assert byte-identical frames.
- `replies/`: pure decoders (`BaseInfo`, `CableInfo`, `DfuInfo`,
  `DongleBaseInfo`, `BondInfo`); `BaseSnapshot` is whichever form the transport
  returns.
- `transport/`: `DarmosharkDevice`, the only `hidapi` user.
- `configurator/`: `MouseConfigurator` composes builders + device into the
  high-level operations; everything goes through it, never around it.
- `profile/`: `DeviceProfile` embeds `reference/darmoshark-m3-profile.json`
  (DPI range, LED colours per level, polling rates, lift-off steps).
- `tests/oracle/fixtures/python_oracle.json` holds frames and decodes recorded
  from the removed Python implementation; the output must still match it byte
  for byte. Regenerating it is no longer possible — change it by hand only
  with hardware evidence, and say so in the commit.

**`crates/cli`** — the `dms` binary, clap over `MouseConfigurator`.
`arguments/` holds the clap definition, `commands/` returns the text each
command prints (`offline_`, `read_` and `write_commands`), so output is tested
without a device. `tests/oracle/fixtures/python_cli.json` froze what the Python
CLI printed.

**`app/`** — Tauri 2, a menu bar app (`ActivationPolicy::Accessory`: no Dock
icon; the window starts hidden and closing it only hides it). `src-tauri/` is
thin: `DeviceGate` opens a fresh configurator per operation and serialises them
(macOS opens the interface exclusively, and unplugging must not wedge the app),
and `read_device_state` is the one read both surfaces use; `dto/` is what
crosses to the webview, in camelCase; `commands/` holds the
`#[tauri::command]`s, and every write redraws the menu bar. `tray/` is the
status item: `MenuBar` (title `25% · 3200`, DPI and rate menu, 30 s poll, all
HID off the main thread), `MenuBarText` (pure, tested), `MenuBarLabels` (copy),
`MenuBarCache` (last reading, so a click redraws at once). A menu write emits
`device-changed`, which the window listens for. `DarmosharkError::Asleep` is
status 2, so both surfaces can say "move the mouse" without matching text.
`src/` is React + Zustand (`useState` is banned by lint): `store/use_mouse_store.ts`
is the single store, `services/mouse_service.ts` the only `invoke` caller,
`routes/app_routes.ts` every command name, `labels/labels.ts` all copy
(Portuguese). `design-system/` components are string-free and take text as
props; `features/configurator/` binds them to the store. Tokens live in
`theme/tokens.css`, every size even. The reset confirmation is a bottom sheet.

## Two transports, one payload

The critical asymmetry to keep in mind when touching `darmoshark_device.rs` or
`mouse_configurator.rs`:

Both live on usage page `0x8C` and are distinguished by product id: the mouse
itself is `0xFF12`, the receiver `0xFF30` (`DarmosharkProtocol::dongleProductIds`,
read through `DarmosharkDevice::uses_dongle_transport`). The receiver has no
vendor usage page of its own, and the reports `0xB3`/`0xB5` that the `dms`
contract talks about exist on neither interface — writing them reaches nothing.

| | Cable (feature `0x52`, 64 B) | Receiver (feature `0x51`, 20 B) |
|---|---|---|
| Writes | fire-and-forget, silent | fire-and-forget, `0xE4` on input `0x54` |
| Reads | always the identity block, never config | full snapshot, opcode `0x07` |
| Long commands | same `0x52` | feature `0x52`, 64 B (button reads) |
| DPI levels | max 5 (`0x40`; extended `0x44` ignored) | max 5, no route for `0x44` |
| Bootloader | the mouse's | the **receiver's** own |

Reading over the receiver has traps, all encoded in `request_dongle`:

- The `0xE4` status is `0` or `4` while pending, `1` when the reply is ready,
  and **`2` when the mouse is asleep** — moving it or clicking wakes it; no
  replug needed.
- The feature buffer holds the *previous* reply until the new one lands, and
  two reads of the same opcode echo the same bytes. "Ready" cannot be trusted
  either: about one read in ten never posts it. So when the buffer already
  echoes the command, `request_dongle` first sends a `primer` read with a
  different echo (bond, or snapshot for a bond request, or another slot), then
  polls the buffer until the real echo appears. Acks are only watched for `2`.
  Commands that address a slot pass `echo_bytes = 2` (all buttons share one
  opcode).
- The ack names no command, so the input queue is drained before every send.
  A receiver write returns once queued; the mouse applies it up to ~200 ms
  later, so `MouseConfigurator::settle_on_receiver` reads it back (at most ten
  reads, never an error) before a write returns.

`setReportRate` is `[65, index, index]`: the vendor contracts name the index
`level`, which is how the old builder mistook it for the DPI level. The index is
the position in the profile's 125/500/1000 Hz list, measured by timing the
mouse's own input reports (8/2/1 ms). Do not trust negative results from before
the transport fix; they were stale reads.
The sleep timer is cable only: the vendor's receiver contract never sends
opcode 10, so `write_sleep_timer` refuses over the receiver. Lift-off shares
its packet with five sensor switches with mixed encodings (`SensorToggles`);
`write_lift_off` resends the stored ones over the receiver instead of the
vendor defaults.

`MouseConfigurator::send_acknowledged` skips ACK checking on both hardware
transports, since neither acknowledges a write the way the `dms` contract
describes. A silent device means a *successful* write, not a failure — do not
add error handling that treats it as one.

## Constraints

- Never widen the DPI range or level count past `DarmosharkProtocol` limits — a
  malformed packet can leave the mouse in an inconsistent onboard profile.
- Reads that need the receiver must fail with a message saying so, not silently
  return the identity block.
- `research/` is gitignored vendor bundle scratch; `reference/` holds the public
  vendor JSON definitions and is tracked.
- Docs are bilingual: any change to `README.md`/`PROTOCOL.md` must be mirrored in
  the `.pt-BR.md` counterpart.
