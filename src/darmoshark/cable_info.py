"""Decoder for the identify reply available over the charging cable."""


class CableInfo:
    """Device identity and battery, read through feature report 0x51.

    This is the only channel the mouse exposes over the cable. It answers
    opcode 0x06 with identity plus battery, but carries no DPI state --
    that lives on the 2.4GHz config interface (usage page 0xFFC1).

    Reply layout (after the echoed report id):

        [0]     opcode echo (0x06)
        [1]     status (1 = ok)
        [3..4]  vendor id, little-endian
        [5..6]  product id, little-endian
        [7..8]  firmware version, little-endian (0x0209 -> 2.0.9)
        [10]    battery percentage
    """

    def __init__(self, vendorId, productId, firmwareVersion, batteryPercent, raw):
        self.vendorId = vendorId
        self.productId = productId
        self.firmwareVersion = firmwareVersion
        self.batteryPercent = batteryPercent
        self.raw = raw

    @staticmethod
    def parse(reply):
        if len(reply) < 12:
            raise ValueError(f"identify reply too short: {len(reply)} bytes")

        body = reply[1:]
        if body[0] != 0x06:
            raise ValueError(f"unexpected opcode echo 0x{body[0]:02X}, expected 0x06")
        if body[1] != 1:
            raise ValueError(f"device reported status {body[1]}, expected 1")

        raw = body[7] | body[8] << 8
        return CableInfo(
            vendorId=body[3] | body[4] << 8,
            productId=body[5] | body[6] << 8,
            firmwareVersion=f"{raw >> 8}.{(raw >> 4) & 0xF}.{raw & 0xF}",
            batteryPercent=body[10],
            raw=bytes(reply),
        )
