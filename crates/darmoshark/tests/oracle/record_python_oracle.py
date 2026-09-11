"""Records what the Python builders and decoders produce, as the Rust oracle.

Run from the repository root, while the Python implementation still exists:

    PYTHONPATH=src .venv/bin/python crates/darmoshark/tests/oracle/record_python_oracle.py \
        > crates/darmoshark/tests/oracle/fixtures/python_oracle.json
"""

import json
import sys

from darmoshark.base_info import BaseInfo
from darmoshark.button_packet import ButtonPacket
from darmoshark.cable_info import CableInfo
from darmoshark.dfu_info import DfuInfo
from darmoshark.dongle_base_info import DongleBaseInfo
from darmoshark.dpi_packet import DpiPacket
from darmoshark.profile_packet import ProfilePacket
from darmoshark.report_rate_packet import ReportRatePacket
from darmoshark.tuning_packet import TuningPacket

frames = []


def frame(builder, args, packet):
    reportId, payload = packet
    frames.append({"builder": builder, "args": args,
                   "reportId": reportId, "payload": payload.hex()})


for values, level, enabled in (
        ([400, 800, 1600, 3200, 4800], 2, None),
        ([400, 800, 1200, 1600, 2400, 3200], 0, None),
        ([800, 1600], 1, 1),
        ([50], 0, None),
        ([26000] * 8, 7, None),
        ([1234, 25999, 51], 2, 2)):
    frame("dpi", [values, level, enabled], DpiPacket.build(values, level, enabled))

for hertz in ReportRatePacket.supportedRates:
    frame("rate", [hertz], ReportRatePacket.build(hertz))

for ms in (0, 8, 20):
    frame("debounce", [ms], TuningPacket.buildDebounce(ms))

for args in ((2, 1, 2, 1, 1, 1), (1, 0, 0, 0, 0, 0), (1, 1, 2, 1, 1, 1), (2, 255, 3, 4, 5, 6)):
    frame("sensor", list(args), TuningPacket.buildSensor(*args))

for args in ((3, 1, 2), (255, 0, 255)):
    frame("scroll", list(args), TuningPacket.buildScroll(*args))

for minutes, mode in ((10, "set"), (0, "get"), (255, "set")):
    frame("sleep", [minutes, mode], TuningPacket.buildSleep(minutes, mode))

for index in range(4):
    frame("profileSwitch", [index], ProfilePacket.buildSwitch(index))
frame("recovery", [1, 2, 3], ProfilePacket.buildRecovery(1, 2, 3))
frame("factoryReset", [], ProfilePacket.buildFactoryReset())

for index in range(5):
    frame("buttonRead", [index], ButtonPacket.buildRead(index))
for index, kind, data in ((1, "dpi", []), (4, "macro", [7, 7, 255]),
                          (0, "keyboard", [0, 4]), (2, "profileSwitch", list(range(59)))):
    frame("buttonWrite", [index, kind, data], ButtonPacket.buildWrite(index, kind, data))

decoders = []

snapshot = bytes.fromhex("510700131303900120034006800cc0123505080000")
info = DongleBaseInfo.parse(snapshot)
decoders.append({"decoder": "dongleBaseInfo", "input": snapshot.hex(), "fields": {
    "profile": info.profile, "dpiLevels": list(info.dpiLevels),
    "activeLevel": info.activeLevel, "reportRate": info.reportRate,
    "debounceMs": info.debounceMs, "sleepMinutes": info.sleepMinutes,
    "liftOff": info.liftOff, "wave": info.wave, "line": info.line,
    "motion": info.motion, "scroll": info.scroll, "eSports": info.eSports}})

base = bytes(range(0x10, 0x10 + 32))
base = bytearray(base)
base[16] = 7
base[19] = 0x80 | 64
info = BaseInfo.parse(bytes(base))
decoders.append({"decoder": "baseInfo", "input": bytes(base).hex(), "fields": {
    "profile": info.profile, "dpiLevels": list(info.dpiLevels),
    "activeLevel": info.activeLevel, "reportRate": info.reportRate,
    "batteryPercent": info.batteryPercent, "batteryCharging": info.batteryCharging,
    "sleepMinutes": info.sleepMinutes}})

identify = bytes.fromhex("5106010000" "8a24" "30ff" "0902" "00" "57" "0000000000000000")
info = CableInfo.parse(identify)
decoders.append({"decoder": "cableInfo", "input": identify.hex(), "fields": {
    "vendorId": info.vendorId, "productId": info.productId,
    "firmwareVersion": info.firmwareVersion, "batteryPercent": info.batteryPercent}})

payload = b"\x00\x00\x00\x00" + b"UCFRF001\x00\x00" + b"\x00\x00" \
    + b"2.0.9r\x00\x00\x00\x00" + b"HW1.2\x00\x00\x00\x00\x00" + b"\x00" * 4
first = bytes([0xAA, 0x55, len(payload), (~len(payload)) & 0xFF, 0x60]) + payload[:27]
rest = payload[27:] + b"\xEE" * 8
info = DfuInfo.parse([first, rest])
decoders.append({"decoder": "dfuInfo", "input": [first.hex(), rest.hex()], "fields": {
    "moduleModel": info.moduleModel, "firmwareVersion": info.firmwareVersion,
    "hardwareVersion": info.hardwareVersion, "raw": info.raw.hex()}})

accented = bytearray(payload)
accented[4:8] = b"UC\xc3\xa9"
accented[16:18] = b"\xff\x80"
accentedFirst = bytes([0xAA, 0x55, len(accented), (~len(accented)) & 0xFF, 0x60]) \
    + bytes(accented[:27])
accentedRest = bytes(accented[27:])
info = DfuInfo.parse([accentedFirst, accentedRest])
decoders.append({"decoder": "dfuInfo", "input": [accentedFirst.hex(), accentedRest.hex()],
                 "fields": {"moduleModel": info.moduleModel,
                            "firmwareVersion": info.firmwareVersion,
                            "hardwareVersion": info.hardwareVersion, "raw": info.raw.hex()}})

json.dump({"source": "generated from the Python implementation (src/darmoshark)",
           "frames": frames, "decoders": decoders}, sys.stdout, indent=2)
sys.stdout.write("\n")
