"""Decoder for the base-info reply (opcode 0x06)."""


class BaseInfo:
    """Parsed snapshot of the mouse configuration.

    Field offsets follow handleInfo() in the official configurator bundle.
    """

    def __init__(self, profile, dpiLevels, activeLevel, reportRate, batteryPercent,
                 batteryCharging, sleepMinutes, raw):
        self.profile = profile
        self.dpiLevels = dpiLevels
        self.activeLevel = activeLevel
        self.reportRate = reportRate
        self.batteryPercent = batteryPercent
        self.batteryCharging = batteryCharging
        self.sleepMinutes = sleepMinutes
        self.raw = raw

    @staticmethod
    def parse(data):
        if len(data) < 30:
            raise ValueError(f"base-info reply too short: {len(data)} bytes")

        gears = data[16]
        if not 1 <= gears <= 8:
            raise ValueError(f"implausible level count in reply: {gears}")

        pairs = ((6, 5), (8, 7), (10, 9), (12, 11), (14, 13),
                 (21, 20), (23, 22), (25, 24))[:gears]
        dpiLevels = tuple(data[high] << 8 | data[low] for high, low in pairs)

        # Byte 3 is the 2.4GHz slot: high nibble report rate, low nibble dpi index.
        rfByte = data[3]
        activeLevel = rfByte & 0x0F
        reportRate = (rfByte >> 4) & 0x0F

        powerByte = data[19]
        return BaseInfo(
            profile=data[1],
            dpiLevels=dpiLevels,
            activeLevel=activeLevel,
            reportRate=reportRate,
            batteryPercent=powerByte & 0x7F,
            batteryCharging=bool(powerByte & 0x80),
            sleepMinutes=data[18],
            raw=bytes(data),
        )
