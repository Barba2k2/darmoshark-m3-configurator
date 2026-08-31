"""Command line front-end for the Darmoshark M3 configurator."""

import argparse
import sys

from darmoshark.button_packet import ButtonPacket
from darmoshark.device import DarmosharkDevice
from darmoshark.device_profile import DeviceProfile
from darmoshark.mouse_configurator import MouseConfigurator
from darmoshark.protocol import DarmosharkProtocol
from darmoshark.report_rate_packet import ReportRatePacket


class Cli:
    """Parses arguments and dispatches to the configurator."""

    @staticmethod
    def buildParser():
        parser = argparse.ArgumentParser(
            prog="dms",
            description="Configure a Darmoshark M3 (Attack Shark M3) mouse.",
            epilog="Writes work over the cable (feature report 0x52) and over the "
                   "2.4GHz dongle. Reads marked [dongle] need the receiver: the "
                   "cable interface always answers with the identity block.",
        )
        sub = parser.add_subparsers(dest="command", required=True)

        sub.add_parser("list", help="list the HID interfaces exposed by the mouse")
        sub.add_parser("capabilities", help="what this model supports (offline)")
        sub.add_parser("colors", help="LED colour legend for DPI and polling rate")
        sub.add_parser("battery", help="identity and battery (cable)")
        sub.add_parser("dfu", help="bootloader, firmware and hardware revision (cable)")
        sub.add_parser("info", help="[dongle] full configuration snapshot")
        sub.add_parser("buttons", help="[dongle] current button assignments")
        sub.add_parser("reset", help="restore factory defaults")

        dpi = sub.add_parser("dpi", help="program the DPI levels")
        dpi.add_argument("values", type=int, nargs="+",
                         help=f"{DarmosharkProtocol.dpiMinimum}-"
                              f"{DarmosharkProtocol.dpiMaximum} per level")
        dpi.add_argument("--active", type=int, default=0,
                         help="index of the level to activate (default: 0)")

        use = sub.add_parser("use", help="switch DPI level (reprograms with profile defaults)")
        use.add_argument("level", type=int)

        rate = sub.add_parser("rate", help="set the polling rate per level (affects wireless)")
        rate.add_argument("values", type=int, nargs="+",
                          help=f"one of {ReportRatePacket.supportedRates} per level")
        rate.add_argument("--active", type=int, default=0)

        debounce = sub.add_parser("debounce", help="click debounce, in ms")
        debounce.add_argument("milliseconds", type=int)

        lod = sub.add_parser("lod", help="lift-off distance")
        lod.add_argument("value", type=int, choices=(1, 2),
                         help="1 = low, 2 = high")

        sleep = sub.add_parser("sleep", help="idle sleep timer, in minutes")
        sleep.add_argument("minutes", type=int)

        profile = sub.add_parser("profile", help="switch onboard profile")
        profile.add_argument("index", type=int)

        button = sub.add_parser("button", help="remap a button")
        button.add_argument("index", type=int)
        button.add_argument("kind", choices=sorted(ButtonPacket.kinds))
        button.add_argument("data", type=int, nargs="*", default=[],
                            help="optional assignment bytes")

        return parser

    @staticmethod
    def main(argv=None):
        args = Cli.buildParser().parse_args(argv)
        handlers = {
            "list": lambda: Cli._listInterfaces(),
            "capabilities": lambda: Cli._showCapabilities(),
            "colors": lambda: Cli._showColorLegend(),
            "battery": lambda: Cli._withMouse(Cli._showCableInfo),
            "dfu": lambda: Cli._withMouse(Cli._showDfuInfo),
            "info": lambda: Cli._withMouse(Cli._showBaseInfo),
            "buttons": lambda: Cli._withMouse(Cli._showButtons),
            "reset": lambda: Cli._withMouse(Cli._resetDefaults),
            "dpi": lambda: Cli._withMouse(Cli._setDpi, args.values, args.active),
            "use": lambda: Cli._withMouse(Cli._useLevel, args.level),
            "rate": lambda: Cli._withMouse(Cli._setRates, args.values, args.active),
            "debounce": lambda: Cli._withMouse(Cli._setDebounce, args.milliseconds),
            "lod": lambda: Cli._withMouse(Cli._setLiftOff, args.value),
            "sleep": lambda: Cli._withMouse(Cli._setSleep, args.minutes),
            "profile": lambda: Cli._withMouse(Cli._setProfile, args.index),
            "button": lambda: Cli._withMouse(
                Cli._setButton, args.index, args.kind, args.data),
        }
        try:
            return handlers[args.command]()
        except (RuntimeError, ValueError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1

    @staticmethod
    def _withMouse(action, *arguments):
        with DarmosharkDevice.open() as device:
            return action(MouseConfigurator(device), *arguments)

    @staticmethod
    def _listInterfaces():
        entries = DarmosharkDevice.discover()
        if not entries:
            print("no Darmoshark interface found.")
            return 1
        for entry in entries:
            print(f"{entry['path'].decode(errors='replace')}  "
                  f"vid=0x{entry['vendor_id']:04X} pid=0x{entry['product_id']:04X}  "
                  f"iface={entry['interface_number']}  "
                  f"usagePage=0x{entry['usage_page']:04X} "
                  f"usage=0x{entry['usage']:02X}  {entry.get('product_string')!r}")
        return 0

    @staticmethod
    def _showCapabilities():
        profile = DeviceProfile.load()
        low, high = profile.dpiRange
        print(f"model          : {profile.name}")
        print(f"dpi range      : {low}-{high}")
        print(f"default levels : {', '.join(str(v) for v in profile.dpiLevels)}")
        print(f"polling rates  : "
              f"{', '.join(str(hz) for hz, _ in profile.reportRates)} Hz")
        print(f"lift-off steps : "
              f"{', '.join(f'{i}={name.split(chr(46))[-1]}' for i, name in profile.liftOffSteps)}")
        print(f"buttons        : {profile.buttonCount}")
        print(f"rgb lighting   : "
              f"{'yes' if profile.hasConfigurableLighting else 'no (indicator LED only)'}")
        return 0

    @staticmethod
    def _showColorLegend():
        profile = DeviceProfile.load()
        names = {
            "#ff0000": "red", "#0060ff": "blue", "#12ff00": "green",
            "#e218ff": "magenta", "#f9e14c": "yellow",
        }
        print("The indicator LED encodes the active setting. Colours are fixed in")
        print("firmware -- they are not configurable.\n")
        print("DPI level:")
        for index, dpi in enumerate(profile.dpiLevels):
            colour = profile.dpiColors[index] if index < len(profile.dpiColors) else "?"
            print(f"  level {index}  {dpi:>5} DPI   {colour}  {names.get(colour, '')}")
        print("\nPolling rate:")
        for hertz, colour in profile.reportRates:
            print(f"  {hertz:>4} Hz          {colour}  {names.get(colour, '')}")
        return 0

    @staticmethod
    def _showCableInfo(configurator):
        info = configurator.readCableInfo()
        print(f"device   : 0x{info.vendorId:04X}:0x{info.productId:04X}")
        print(f"firmware : {info.firmwareVersion}")
        print(f"battery  : {info.batteryPercent}%")
        return 0

    @staticmethod
    def _showDfuInfo(configurator):
        info = configurator.readDfuInfo()
        print(f"module   : {info.moduleModel}")
        print(f"firmware : {info.firmwareVersion}")
        print(f"hardware : {info.hardwareVersion}")
        return 0

    @staticmethod
    def _showBaseInfo(configurator):
        info = configurator.readBaseInfo()
        rate = ReportRatePacket.codeToRate(info.reportRate)
        print(f"profile      : {info.profile}")
        print(f"dpi levels   : {', '.join(str(v) for v in info.dpiLevels)}")
        print(f"active level : {info.activeLevel}")
        print(f"polling rate : {rate if rate else f'code {info.reportRate}'} Hz")
        print(f"battery      : {info.batteryPercent}%"
              f"{' (charging)' if info.batteryCharging else ''}")
        print(f"sleep        : {info.sleepMinutes} min")
        return 0

    @staticmethod
    def _showButtons(configurator):
        for entry in configurator.readAllButtons():
            print(f"button {entry['button']}: {entry['kind']}"
                  f" (0x{entry['kindValue']:02X})")
        return 0

    @staticmethod
    def _resetDefaults(configurator):
        configurator.restoreFactoryDefaults()
        print("factory defaults restored")
        return 0

    @staticmethod
    def _setDpi(configurator, values, active):
        configurator.writeDpiLevels(values, active)
        print(f"programmed {len(values)} level(s): "
              f"{', '.join(str(v) for v in values)}; active index {active}")
        return 0

    @staticmethod
    def _useLevel(configurator, level):
        try:
            info = configurator.selectDpiLevel(level)
            print(f"switched to level {level} ({info.dpiLevels[level]} DPI)")
            return 0
        except (RuntimeError, ValueError):
            pass

        # Cable transport cannot read the stored levels back, so reprogram
        # from the vendor profile defaults and say so.
        levels = DeviceProfile.load().dpiLevels
        if not 0 <= level < len(levels):
            raise ValueError(f"level {level} out of range, profile has {len(levels)}")
        configurator.writeDpiLevels(levels, level)
        print(f"switched to level {level} ({levels[level]} DPI)")
        print("note: levels could not be read back over the cable, so they were "
              "rewritten with the profile defaults "
              f"({', '.join(str(v) for v in levels)}).")
        return 0

    @staticmethod
    def _setRates(configurator, values, active):
        configurator.writeReportRates(values, active)
        print(f"polling rates set to {', '.join(str(v) for v in values)} Hz")
        return 0

    @staticmethod
    def _setDebounce(configurator, milliseconds):
        configurator.writeDebounce(milliseconds)
        print(f"debounce set to {milliseconds} ms")
        return 0

    @staticmethod
    def _setLiftOff(configurator, value):
        configurator.writeSensorSettings(liftOff=value)
        print(f"lift-off distance set to {value}")
        return 0

    @staticmethod
    def _setSleep(configurator, minutes):
        configurator.writeSleepTimer(minutes)
        print(f"sleep timer set to {minutes} min")
        return 0

    @staticmethod
    def _setProfile(configurator, index):
        configurator.switchProfile(index)
        print(f"switched to profile {index}")
        return 0

    @staticmethod
    def _setButton(configurator, index, kind, data):
        configurator.writeButton(index, kind, data)
        print(f"button {index} assigned to {kind}"
              f"{' with data ' + str(data) if data else ''}")
        return 0


if __name__ == "__main__":
    sys.exit(Cli.main())
