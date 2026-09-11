"""Decoder for the configuration snapshot read through the 2.4GHz receiver."""


class DongleBaseInfo:
    """Stored configuration as the receiver reports it (opcode 0x07).

    This is the read the charging cable cannot do: over the cable every opcode
    comes back as the identity block, so the mouse never tells what it holds.

    Offsets follow getBaseInfo() of the "M" contract in the official bundle,
    counted from the opcode echo (the report id is stripped first):

        [0]      opcode echo (0x07)
        [1]      active onboard profile
        [2..4]   usb / 2.4GHz / bluetooth slots -- low nibble the active dpi
                 index, high nibble the report-rate index
        [5..14]  five little-endian uint16 dpi values
        [15]     sensor and system bits
        [16]     number of enabled dpi levels
        [17]     click debounce, in milliseconds
        [18]     sleep timer, in minutes
    """

    opcode = 0x07

    def __init__(self, profile, dpiLevels, activeLevel, reportRate, debounceMs,
                 sleepMinutes, liftOff, wave, line, motion, scroll, eSports, raw):
        self.profile = profile
        self.dpiLevels = dpiLevels
        self.activeLevel = activeLevel
        self.reportRate = reportRate
        self.debounceMs = debounceMs
        self.sleepMinutes = sleepMinutes
        self.liftOff = liftOff
        self.wave = wave
        self.line = line
        self.motion = motion
        self.scroll = scroll
        self.eSports = eSports
        self.raw = raw

    # Battery does not travel in this reply; it comes from the identify read.
    batteryPercent = None
    batteryCharging = None

    @staticmethod
    def parse(reply):
        if len(reply) < 20:
            raise ValueError(f"snapshot reply too short: {len(reply)} bytes")

        body = reply[1:]
        if body[0] != DongleBaseInfo.opcode:
            raise ValueError(
                f"unexpected opcode echo 0x{body[0]:02X}, expected "
                f"0x{DongleBaseInfo.opcode:02X}")

        gears = body[16]
        if not 1 <= gears <= 8:
            raise ValueError(f"implausible level count in reply: {gears}")

        pairs = ((6, 5), (8, 7), (10, 9), (12, 11), (14, 13),
                 (21, 20), (23, 22), (25, 24))[:gears]
        if pairs[-1][0] >= len(body):
            raise ValueError(
                f"reply holds {len(body)} bytes, too few for {gears} dpi levels")

        # Byte 3 is the 2.4GHz slot, the one in use while this read is possible.
        radioByte = body[3]
        sensorByte = body[15]

        return DongleBaseInfo(
            profile=body[1],
            dpiLevels=tuple(body[high] << 8 | body[low] for high, low in pairs),
            activeLevel=radioByte & 0x0F,
            reportRate=(radioByte >> 4) & 0x0F,
            debounceMs=body[17],
            sleepMinutes=body[18],
            liftOff=sensorByte & 0x03,
            wave=(sensorByte >> 2) & 1,
            line=(sensorByte >> 3) & 1,
            motion=(sensorByte >> 4) & 1,
            scroll=(sensorByte >> 6) & 1,
            eSports=(sensorByte >> 7) & 1,
            raw=bytes(reply),
        )
