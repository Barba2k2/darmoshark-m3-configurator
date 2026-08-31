"""Bridges the GUI to the mouse, keeping HID details out of the widgets."""

from darmoshark.device import DarmosharkDevice
from darmoshark.device_profile import DeviceProfile
from darmoshark.mouse_configurator import MouseConfigurator


class MouseService:
    """Every hardware call the window needs, each one self contained.

    Connections are opened per operation rather than held open, so unplugging
    the mouse mid-session cannot leave the window wedged on a dead handle.
    """

    def __init__(self):
        self.profile = DeviceProfile.load()

    @property
    def isConnected(self):
        return bool(DarmosharkDevice.discover())

    def readIdentity(self):
        with DarmosharkDevice.open() as device:
            return MouseConfigurator(device).readCableInfo()

    def readFirmware(self):
        with DarmosharkDevice.open() as device:
            return MouseConfigurator(device).readDfuInfo()

    def applyDpiLevels(self, values, activeLevel):
        with DarmosharkDevice.open() as device:
            MouseConfigurator(device).writeDpiLevels(values, activeLevel)

    def applyReportRate(self, hertz, levelCount, activeLevel):
        with DarmosharkDevice.open() as device:
            MouseConfigurator(device).writeReportRates(
                [hertz] * levelCount, activeLevel)

    def applyLiftOff(self, value):
        with DarmosharkDevice.open() as device:
            MouseConfigurator(device).writeSensorSettings(liftOff=value)

    def applyDebounce(self, milliseconds):
        with DarmosharkDevice.open() as device:
            MouseConfigurator(device).writeDebounce(milliseconds)

    def applySleepTimer(self, minutes):
        with DarmosharkDevice.open() as device:
            MouseConfigurator(device).writeSleepTimer(minutes)

    def restoreDefaults(self):
        with DarmosharkDevice.open() as device:
            MouseConfigurator(device).restoreFactoryDefaults()
