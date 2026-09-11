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

```bash
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

## Usage

### Graphical interface

```bash
PYTHONPATH=src .venv/bin/python src/gui/app.py
```

Five DPI levels, each carrying the real colour of the mouse indicator LED, free
value fields from 50 to 26000, polling rate, lift-off distance, debounce and
sleep timer.

### Command line

```bash
PYTHONPATH=src .venv/bin/python src/cli.py <command>
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
| `rate 1000 1000 1000 1000 1000` | polling rate (affects wireless modes) | ✅ |
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
| Polling rate | ❔ unverified | ❔ unverified |
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
src/darmoshark/    protocol, packet builders, decoders, HID transport
src/gui/           PySide6 interface (one widget per file)
src/cli.py         command line interface
tests/             packet encoding tests
reference/         public vendor definitions for this model
```

## Tests

```bash
PYTHONPATH=src .venv/bin/python -m unittest discover -s tests
```

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
