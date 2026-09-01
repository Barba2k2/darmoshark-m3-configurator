"""High level configuration operations for a connected Darmoshark mouse."""

from darmoshark.base_info import BaseInfo
from darmoshark.button_packet import ButtonPacket
from darmoshark.cable_info import CableInfo
from darmoshark.dfu_info import DfuInfo
from darmoshark.dms_commands import DmsCommands
from darmoshark.dongle_base_info import DongleBaseInfo
from darmoshark.dpi_packet import DpiPacket
from darmoshark.profile_packet import ProfilePacket
from darmoshark.protocol import DarmosharkProtocol
from darmoshark.report_rate_packet import ReportRatePacket
from darmoshark.tuning_packet import TuningPacket


class MouseConfigurator:
    """Every setting the official configurator can change, over the dms channel.

    Writes work over either transport. Reading the stored configuration back
    needs the 2.4GHz receiver -- over the cable every opcode answers with the
    identity block instead.
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

    # -- reads that need the 2.4GHz receiver -------------------------------

    def readDongleBaseInfo(self):
        """Reads the stored configuration back, receiver only."""
        payload = bytearray(DarmosharkProtocol.donglePayloadSize)
        payload[0] = DarmosharkProtocol.cmdDongleBaseInfo
        reply = self.device.requestDongle(bytes(payload))
        if reply is None:
            raise RuntimeError("receiver did not answer the configuration read")
        return DongleBaseInfo.parse(reply)

    def readBondInfo(self):
        """Identity of the mouse the receiver is linked to, and the link state."""
        payload = bytearray(DarmosharkProtocol.donglePayloadSize)
        payload[0] = DmsCommands.getBondInfo
        reply = self.device.requestDongle(bytes(payload))
        if reply is None:
            raise RuntimeError("receiver did not answer the bond request")
        body = reply[1:]
        return {
            "vendorId": body[2] | body[3] << 8,
            "productId": body[4] | body[5] << 8,
            "linked": bool(body[6]),
        }

    # -- reads that need the config interface ------------------------------

    def readBaseInfo(self):
        if self.device.usesDongleTransport:
            return self.readDongleBaseInfo()

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
        if self.device.usesDongleTransport:
            # The opcode alone does not identify the answer here: every button
            # shares it, so the index has to be echoed back too.
            reply = self.device.requestDongle(payload, echoBytes=2)
            reply = reply[1:] if reply else None
        else:
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

        Both hardware transports issue writes fire-and-forget, the same way the
        vendor software does: the cable stays silent, and the receiver answers
        only that it queued the command. There is nothing to verify in either
        case, so a missing reply is not an error.
        """
        if self.device.usesCableTransport or self.device.usesDongleTransport:
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
