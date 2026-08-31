"""Decoder for the DFU identity reply (cmd 96) on the cable interface."""


class DfuInfo:
    """Module model, firmware and hardware revision reported by the bootloader.

    The reply is split across several 32-byte input reports. Byte 2 of the
    first one carries the total payload length; the payload itself starts at
    offset 5 and continues at offset 0 of the following reports.

    Field offsets inside the reassembled payload:

        [4..13]   module model, NUL padded ASCII
        [16..25]  firmware version string
        [26..35]  hardware version string
    """

    def __init__(self, moduleModel, firmwareVersion, hardwareVersion, raw):
        self.moduleModel = moduleModel
        self.firmwareVersion = firmwareVersion
        self.hardwareVersion = hardwareVersion
        self.raw = raw

    @staticmethod
    def parse(packets):
        if not packets:
            raise ValueError("no DFU reply received")

        first = packets[0]
        if first[0] != 0xAA or first[1] != 0x55:
            raise ValueError(
                f"bad DFU header {first[0]:02X} {first[1]:02X}, expected AA 55")
        if first[3] != (~first[2]) & 0xFF:
            raise ValueError("DFU length checksum mismatch")

        remaining = first[2]
        payload = bytearray(first[5:])
        remaining -= len(payload)
        for packet in packets[1:]:
            if remaining <= 0:
                break
            chunk = packet[:remaining]
            payload.extend(chunk)
            remaining -= len(chunk)

        if len(payload) < 36:
            raise ValueError(f"DFU payload truncated: {len(payload)} bytes")

        return DfuInfo(
            moduleModel=DfuInfo._text(payload[4:14]),
            firmwareVersion=DfuInfo._text(payload[16:26]),
            hardwareVersion=DfuInfo._text(payload[26:36]),
            raw=bytes(payload),
        )

    @staticmethod
    def _text(chunk):
        return bytes(b for b in chunk if b != 0).decode("ascii", errors="replace")
