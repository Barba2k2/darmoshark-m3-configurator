"""Builder for the polling-rate packet (opcode 65)."""

from darmoshark.dms_commands import DmsCommands
from darmoshark.protocol import DarmosharkProtocol


class ReportRatePacket:
    """Programs the polling rate assigned to each DPI level.

    Layout on the short report:

        [0]     opcode 65
        [1..2]  active level index
        [3..8]  one rate code per level
        [9]     number of enabled levels

    The device stores a rate *code*, not the frequency itself. Codes come from
    the position of the value in the device profile (125/500/1000 Hz).
    """

    supportedRates = (125, 500, 1000)

    @staticmethod
    def build(rateCodes, activeLevel, enabledLevels=None):
        codes = tuple(int(c) for c in rateCodes)
        if not codes:
            raise ValueError("at least one rate code is required")
        if len(codes) > 6:
            raise ValueError(f"at most 6 levels are supported, got {len(codes)}")
        if not 0 <= activeLevel < len(codes):
            raise ValueError(
                f"activeLevel must index one of the {len(codes)} levels, "
                f"got {activeLevel}")
        for code in codes:
            if not 0 <= code <= 0xFF:
                raise ValueError(f"rate code {code} does not fit in a byte")

        gears = len(codes) if enabledLevels is None else int(enabledLevels)
        if not 1 <= gears <= len(codes):
            raise ValueError(
                f"enabledLevels must be between 1 and {len(codes)}, got {gears}")

        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.setReportRate
        payload[1] = activeLevel
        payload[2] = activeLevel
        for index, code in enumerate(codes):
            payload[3 + index] = code
        payload[9] = gears
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
