"""Builder for the DPI configuration packet."""

from darmoshark.protocol import DarmosharkProtocol


class DpiPacket:
    """Builds the byte payload that programs the DPI levels.

    Layout for the short form (<= 5 levels), opcode 0x40 on report 0xB5:

        [0]      opcode 0x40
        [1..3]   currently selected level index (repeated 3x)
        [4..13]  five little-endian uint16 dpi values
        [14]     number of enabled levels

    Every value is validated before it reaches the device: a malformed packet
    can put the mouse into an inconsistent profile.
    """

    @staticmethod
    def build(dpiValues, currentLevel, enabledLevels=None):
        values = tuple(int(v) for v in dpiValues)
        DpiPacket._validate(values, currentLevel)

        gears = len(values) if enabledLevels is None else int(enabledLevels)
        if not 1 <= gears <= len(values):
            raise ValueError(
                f"enabledLevels must be between 1 and {len(values)}, got {gears}"
            )

        if len(values) > DarmosharkProtocol.maxShortLevels:
            return DpiPacket._buildLong(values, currentLevel)
        return DpiPacket._buildShort(values, currentLevel, gears)

    @staticmethod
    def _buildShort(values, currentLevel, gears):
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DarmosharkProtocol.cmdSetDpiShort
        payload[1] = currentLevel
        payload[2] = currentLevel
        payload[3] = currentLevel
        for index, value in enumerate(values):
            payload[4 + index * 2] = value & 0xFF
            payload[5 + index * 2] = (value >> 8) & 0xFF
        payload[14] = gears
        return DarmosharkProtocol.shortReportId, bytes(payload)

    @staticmethod
    def _buildLong(values, currentLevel):
        payload = bytearray(DarmosharkProtocol.longPayloadSize)
        payload[0] = DarmosharkProtocol.cmdSetDpiLong
        payload[1] = currentLevel
        payload[2] = currentLevel
        payload[3] = currentLevel
        payload[4] = len(values)
        for index, value in enumerate(values):
            payload[5 + index * 2] = value & 0xFF
            payload[6 + index * 2] = (value >> 8) & 0xFF
        return DarmosharkProtocol.longReportId, bytes(payload)

    @staticmethod
    def _validate(values, currentLevel):
        if not values:
            raise ValueError("at least one DPI value is required")
        if len(values) > DarmosharkProtocol.maxLevels:
            raise ValueError(
                f"at most {DarmosharkProtocol.maxLevels} DPI levels are supported, "
                f"got {len(values)}"
            )
        for value in values:
            if not DarmosharkProtocol.dpiMinimum <= value <= DarmosharkProtocol.dpiMaximum:
                raise ValueError(
                    f"DPI {value} is out of range "
                    f"({DarmosharkProtocol.dpiMinimum}-{DarmosharkProtocol.dpiMaximum})"
                )
        if not 0 <= currentLevel < len(values):
            raise ValueError(
                f"currentLevel must index one of the {len(values)} values, "
                f"got {currentLevel}"
            )
