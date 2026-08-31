"""High level configuration operations for a connected Darmoshark mouse."""

from darmoshark.base_info import BaseInfo
from darmoshark.button_packet import ButtonPacket
from darmoshark.cable_info import CableInfo
from darmoshark.dfu_info import DfuInfo
from darmoshark.dms_commands import DmsCommands
from darmoshark.dpi_packet import DpiPacket
from darmoshark.profile_packet import ProfilePacket
from darmoshark.protocol import DarmosharkProtocol
from darmoshark.report_rate_packet import ReportRatePacket
from darmoshark.tuning_packet import TuningPacket


class MouseConfigurator:
    """Every setting the official configurator can change, over the dms channel.

    Reads that the cable exposes (identity, battery, bootloader) work anywhere.
    Everything else needs the 2.4GHz config interface.
    """

    def __init__(self, device):
        self.device = device

    # -- reads available over the cable -----------------------------------

    def readCableInfo(self):
        payload = bytearray(DarmosharkProtocol.identifyPayloadSize)
        payload[0] = DarmosharkProtocol.cmdIdentify
        reply = self.device.requestFeature(
            DarmosharkProtocol.identifyFeatureId, bytes(payload))
        if reply is None:
            raise RuntimeError("mouse did not answer the identify request")
        return CableInfo.parse(reply)

    def readDfuInfo(self):
        packets = self.device.requestDfu(1, DarmosharkProtocol.cmdDfuModuleInfo)
        if not packets:
            raise RuntimeError("bootloader did not answer the module-info request")
        return DfuInfo.parse(packets)

    # -- reads that need the config interface ------------------------------

    def readBaseInfo(self):
        payload = bytearray(DarmosharkProtocol.longPayloadSize)
        payload[0] = DmsCommands.getMouseExtInfo

        reply = self.device.requestCommand(
            DarmosharkProtocol.longReportId, bytes(payload))
        if reply is None:
            raise RuntimeError(
                "mouse did not answer the base-info request. The open interface is "
                "probably the cable one — connect through the 2.4GHz dongle.")
        if reply[0] not in DarmosharkProtocol.baseInfoReplyOpcodes:
            raise RuntimeError(
                f"unexpected reply opcode 0x{reply[0]:02X}, expected one of "
                f"{[hex(o) for o in DarmosharkProtocol.baseInfoReplyOpcodes]}")
        return BaseInfo.parse(reply)

    def readButton(self, buttonIndex):
        reportId, payload = ButtonPacket.buildRead(buttonIndex)
        reply = self.device.requestCommand(reportId, payload)
        if reply is None:
            raise RuntimeError(f"no answer reading button {buttonIndex}")
        return ButtonPacket.parseRead(reply, buttonIndex)

    def readAllButtons(self):
        return [self.readButton(index) for index in range(ButtonPacket.buttonCount)]

    # -- writes -------------------------------------------------------------

    def writeDpiLevels(self, dpiValues, activeLevel, enabledLevels=None):
        reportId, payload = DpiPacket.build(dpiValues, activeLevel, enabledLevels)
        self.device.sendCommand(reportId, payload)

    def selectDpiLevel(self, levelIndex):
        info = self.readBaseInfo()
        if not 0 <= levelIndex < len(info.dpiLevels):
            raise ValueError(
                f"level {levelIndex} out of range, mouse has {len(info.dpiLevels)}")
        self.writeDpiLevels(info.dpiLevels, levelIndex, len(info.dpiLevels))
        return info

    def writeReportRates(self, hertzValues, activeLevel, enabledLevels=None):
        codes = [ReportRatePacket.rateToCode(v) for v in hertzValues]
        reportId, payload = ReportRatePacket.build(codes, activeLevel, enabledLevels)
        self._sendAcknowledged(reportId, payload, DmsCommands.setReportRate)

    def writeDebounce(self, milliseconds):
        reportId, payload = TuningPacket.buildDebounce(milliseconds)
        self._sendAcknowledged(reportId, payload, DmsCommands.setButtonDebounce)

    def writeSensorSettings(self, liftOff=1, wave=1, line=2, motion=1,
                            scroll=1, eSports=1):
        reportId, payload = TuningPacket.buildSensor(
            liftOff, wave, line, motion, scroll, eSports)
        self._sendAcknowledged(reportId, payload, DmsCommands.setSensorLiftCutoff)

    def writeScrollSettings(self, speed, inertia, spl):
        reportId, payload = TuningPacket.buildScroll(speed, inertia, spl)
        self._sendAcknowledged(reportId, payload, DmsCommands.setScroll)

    def writeSleepTimer(self, minutes):
        reportId, payload = TuningPacket.buildSleep(minutes, "set")
        self._sendAcknowledged(reportId, payload, DmsCommands.deviceTime)

    def switchProfile(self, profileIndex):
        reportId, payload = ProfilePacket.buildSwitch(profileIndex)
        self._sendAcknowledged(reportId, payload, DmsCommands.profileSwitch)

    def restoreFactoryDefaults(self):
        reportId, payload = ProfilePacket.buildFactoryReset()
        self._sendAcknowledged(reportId, payload, DmsCommands.driverConfigRecovery)

    def writeButton(self, buttonIndex, kind, data=()):
        reportId, payload = ButtonPacket.buildWrite(buttonIndex, kind, data)
        self._sendAcknowledged(reportId, payload, DmsCommands.setButtonConfig)

    # -- internals ----------------------------------------------------------

    def _sendAcknowledged(self, reportId, payload, opcode, required=True):
        """Sends a command and checks the 0xE4 acknowledgement frame.

        Only the dongle transport acknowledges. Over the cable the writes are
        fire-and-forget -- the same way the vendor software issues them -- so
        there is nothing to verify and a missing reply is not an error.
        """
        if self.device.usesCableTransport:
            self.device.sendCommand(reportId, payload)
            return None

        reply = self.device.requestCommand(reportId, payload)
        if reply is None:
            if required:
                raise RuntimeError(
                    f"no acknowledgement for command {opcode}. Is the mouse on the "
                    "2.4GHz config interface?")
            return None
        if reply[0] != DmsCommands.ackOpcode:
            return reply
        if len(reply) > 1 and reply[1] != DmsCommands.ackStatusOk:
            raise RuntimeError(
                f"device rejected command {opcode} with status {reply[1]}")
        return reply
