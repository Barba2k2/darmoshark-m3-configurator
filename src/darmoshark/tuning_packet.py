"""Builders for the sensor and click tuning packets."""

from darmoshark.dms_commands import DmsCommands
from darmoshark.protocol import DarmosharkProtocol


class TuningPacket:
    """Debounce, lift-off distance, scroll and sleep timer packets.

    All of them ride the 20-byte short report (0xB5) and are acknowledged with
    an 0xE4 frame echoing the opcode.
    """

    debounceMinimum = 0
    debounceMaximum = 20
    liftOffValues = (1, 2)          # M3 profile declares only low/high
    sleepMinimumMinutes = 0
    sleepMaximumMinutes = 255

    @staticmethod
    def buildDebounce(milliseconds):
        if not TuningPacket.debounceMinimum <= milliseconds <= TuningPacket.debounceMaximum:
            raise ValueError(
                f"debounce must be {TuningPacket.debounceMinimum}-"
                f"{TuningPacket.debounceMaximum} ms, got {milliseconds}")
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.setButtonDebounce
        payload[1] = milliseconds
        return DarmosharkProtocol.shortReportId, bytes(payload)

    @staticmethod
    def buildSensor(liftOff=1, wave=1, line=2, motion=1, scroll=1, eSports=1):
        """Sensor block: lift-off distance plus the assorted on/off toggles."""
        if liftOff not in TuningPacket.liftOffValues:
            raise ValueError(
                f"liftOff must be one of {TuningPacket.liftOffValues}, got {liftOff}")
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.setSensorLiftCutoff
        payload[1] = liftOff
        payload[2] = wave
        payload[3] = line
        payload[4] = motion
        payload[6] = scroll
        payload[7] = eSports
        return DarmosharkProtocol.shortReportId, bytes(payload)

    @staticmethod
    def buildScroll(speed, inertia, spl):
        for name, value in (("speed", speed), ("inertia", inertia), ("spl", spl)):
            if not 0 <= value <= 0xFF:
                raise ValueError(f"{name} must fit in a byte, got {value}")
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.setScroll
        payload[1] = speed
        payload[2] = inertia
        payload[3] = spl
        return DarmosharkProtocol.shortReportId, bytes(payload)

    @staticmethod
    def buildSleep(minutes, mode="set"):
        modes = {"set": 1, "get": 2}
        if mode not in modes:
            raise ValueError(f"mode must be 'set' or 'get', got {mode!r}")
        if not TuningPacket.sleepMinimumMinutes <= minutes <= TuningPacket.sleepMaximumMinutes:
            raise ValueError(
                f"sleep must be {TuningPacket.sleepMinimumMinutes}-"
                f"{TuningPacket.sleepMaximumMinutes} minutes, got {minutes}")
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.deviceTime
        payload[1] = modes[mode]
        payload[2] = minutes
        return DarmosharkProtocol.shortReportId, bytes(payload)
