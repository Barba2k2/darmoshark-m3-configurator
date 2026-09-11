"""Builder for the polling-rate packet (opcode 65)."""

from darmoshark.dms_commands import DmsCommands
from darmoshark.protocol import DarmosharkProtocol


class ReportRatePacket:
    """Programs the polling rate of the active connection slot.

    Layout on the short report, the same in the cable and receiver contracts:

        [0]     opcode 65
        [1..2]  rate index, repeated

    The index is the position of the frequency in the device profile
    (125/500/1000 Hz) and the same value the snapshot reports in the high
    nibble of the slot byte. Measured on an M3 through the receiver: index 0
    polls every 8 ms, 1 every 2 ms, 2 every 1 ms.
    """

    supportedRates = (125, 500, 1000)

    @staticmethod
    def build(hertz):
        code = ReportRatePacket.rateToCode(hertz)
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.setReportRate
        payload[1] = code
        payload[2] = code
        return DarmosharkProtocol.shortReportId, bytes(payload)

    @staticmethod
    def rateToCode(hertz):
        """Maps a frequency to its profile index, as the configurator does."""
        if hertz not in ReportRatePacket.supportedRates:
            raise ValueError(
                f"unsupported polling rate {hertz} Hz, expected one of "
                f"{ReportRatePacket.supportedRates}")
        return ReportRatePacket.supportedRates.index(hertz)

    @staticmethod
    def codeToRate(code):
        if not 0 <= code < len(ReportRatePacket.supportedRates):
            return None
        return ReportRatePacket.supportedRates[code]
