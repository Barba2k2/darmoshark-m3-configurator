# Darmoshark M3 — open configurator

**English** · [Português](README.pt-BR.md)

Control DPI, polling rate and the remaining settings of the **Darmoshark M3**
mouse (also sold as **Attack Shark M3**) on **macOS**, without the vendor
software and without the 2.4GHz receiver — it works over the USB-C cable.

The protocol was reverse engineered from the official WebHID configurator
(`darmoshark.cc`) and validated against real hardware. No public documentation
of this protocol existed anywhere.

## Why this exists

The vendor software is a WebHID page that depends on servers in China, and
there is no native macOS application. This project delivers the same settings
through a CLI and a native window, working offline.

## Install

Rust, Node and pnpm are the only requirements.

```bash
cargo build --release -p darmoshark-cli
```

```bash
cd app && pnpm install && pnpm tauri build
```

The second one produces `target/release/bundle/macos/Darmoshark M3.app`.

## Usage

### Graphical interface

```bash
cd app && pnpm tauri dev
```

Five DPI levels, each carrying the real colour of the mouse indicator LED, free
value fields from 50 to 26000, polling rate, lift-off distance, debounce and
sleep timer. Through the receiver the window opens with the values stored in
the mouse; through the cable, which cannot read them, with the factory ones.

The app lives in the menu bar, not the Dock: the status item shows battery and
active DPI (`25% · 3200`), and its menu switches DPI level and polling rate or
opens the window. Closing the window keeps it in the menu bar; "Sair" quits.
`--` means no reading — usually the mouse fell asleep; moving it wakes it.

### Command line

```bash
target/release/dms <command>
```

| Command | Description | Cable |
|---|---|---|
| `list` | HID interfaces exposed by the mouse | ✅ |
| `capabilities` | what this model supports (offline) | ✅ |
| `colors` | LED colour legend | ✅ |
| `battery` | identity and battery | ✅ |
| `dfu` | module, firmware and hardware revision | ✅ |
| `dpi 400 800 1600 3200 4800 --active 3` | program the levels | ✅ |
| `use 3` | switch the active level | ✅ |
| `rate 1000` | polling rate: 125, 500 or 1000 Hz | ❔ |
| `debounce 8` | click debounce, in ms | ✅ |
| `lod 1` | lift-off distance (1 low, 2 high) | ✅ |
| `sleep 10` | sleep after N minutes | ✅ |
| `profile 0` | switch the onboard profile | ✅ |
| `button 3 dpi` | remap a button | ✅ |
| `reset` | restore factory defaults | ✅ |
| `info` | full stored configuration | ⚠️ receiver only |
| `buttons` | current button assignments | ⚠️ receiver only |
| `bond` | which mouse the receiver is linked to | ⚠️ receiver only |

Finding the active DPI without the receiver is possible through the LED colour
— `colors` prints the legend.

## Cable or receiver

Both are validated on hardware. The receiver is the one that can read the
mouse's stored configuration back.

| Operation | Cable (feature `0x52`) | 2.4GHz receiver (feature `0x51`) |
|---|---|---|
| Write configuration | ✅ | ✅ |
| Read configuration | ❌ always answers with identity | ✅ |
| Identity, firmware, battery | ✅ | ✅ |
| Up to 5 DPI levels | ✅ | ✅ |
| 6 or more levels | ❌ extended format is ignored | ❌ no route for the long form |
| Polling rate | ❔ cannot be read back | ✅ |
| Sleep timer | ✅ write (cannot be read back) | ❌ not relayed |
| Bootloader read | the mouse's | the **receiver's** own |

The receiver drops its config link silently: the cursor keeps working while
every command answers "no link". Unplugging and replugging it is the only
recovery.

Full protocol details in [PROTOCOL.md](PROTOCOL.md): the three HID channels,
the 31 opcode table, every packet layout, and the channels that do **not**
work.

## Validated hardware

| | |
|---|---|
| Model | Darmoshark M3 (Attack Shark M3) |
| VID:PID | `0x248A:0xFF12` |
| Module | `MOTO_M3` |
| Firmware | `2.0.9r` |
| Hardware | `1.0.0` |
| Sensor | PAW3395, 50–26000 DPI |
| Battery | 500 mAh |
| Receiver | `0x248A:0xFF30`, module `UCFRF001`, fw `e.1.0r-7` |
| System | macOS 26.6 (arm64) |

Other Darmoshark models sharing this protocol may work, but were not tested.

## Layout

```
crates/darmoshark/ protocol, packet builders, decoders, HID transport (Rust)
crates/cli/        `dms`, the command line interface
app/               Tauri window: React + Zustand in src/, commands in src-tauri/
reference/         public vendor definitions for this model
```

## Tests

```bash
cargo test
```

```bash
cd app && pnpm typecheck && pnpm lint && pnpm test
```

With the mouse or receiver plugged in, `DARMOSHARK_HARDWARE=1 cargo test --test
hardware -- --test-threads=1` also exercises the real transport; each write is
read back and then restored.

The tests verify that the generated packets are byte-identical to the ones the
vendor software builds, plus the range validations.

## Reproducing the reverse engineering

The protocol came out of the official configurator's JavaScript bundle:

```bash
curl -s https://www.darmoshark.cc/ -o index.html
# download the referenced main.*.js and beautify it (jsbeautifier)
```

The public definition of this model lives in `reference/`, fetched from:

- `https://launcher.keychron.com/vapi/v2/product/613089042`
- `https://launcher.keychron.com/static/device/613089042/json/v3.json`

`613089042` is the `vpId`, computed as `vid << 16 | pid`.

## Disclaimer

Independent project, not affiliated with Darmoshark, Attack Shark, Motospeed or
Keychron. Writing configuration to a USB device carries risk: use at your own.
The `reset` command restores factory defaults if anything ends up misplaced.

## License

MIT
