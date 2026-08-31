"""Builders and decoder for the button remapping commands."""

from darmoshark.dms_commands import DmsCommands
from darmoshark.protocol import DarmosharkProtocol


class ButtonPacket:
    """Reads and writes the per-button assignment.

    Assignment kinds, as enumerated by the configurator:

        0 remove   1 Mouse    2 Keyboard  3 Media   4 Macro
        5 Dpi      6 Light    7 GameReinforce      8 ShortCut
        9 disable  10 profileSwitch
    """

    kinds = {
        "remove": 0, "mouse": 1, "keyboard": 2, "media": 3, "macro": 4,
        "dpi": 5, "light": 6, "gameReinforce": 7, "shortCut": 8,
        "disable": 9, "profileSwitch": 10,
    }

    buttonCount = 5

    @staticmethod
    def buildRead(buttonIndex):
        ButtonPacket._validateIndex(buttonIndex)
        payload = bytearray(DarmosharkProtocol.longPayloadSize)
        payload[0] = DmsCommands.getButtonConfig
        payload[1] = buttonIndex
        return DarmosharkProtocol.longReportId, bytes(payload)

    @staticmethod
    def buildWrite(buttonIndex, kind, data=()):
        ButtonPacket._validateIndex(buttonIndex)
        kindValue = ButtonPacket.resolveKind(kind)
        extra = tuple(int(b) for b in data)
        if len(extra) > DarmosharkProtocol.longPayloadSize - 4:
            raise ValueError(f"assignment data too long: {len(extra)} bytes")
        for byte in extra:
            if not 0 <= byte <= 0xFF:
                raise ValueError(f"assignment byte {byte} does not fit in a byte")

        payload = bytearray(DarmosharkProtocol.longPayloadSize)
        payload[0] = DmsCommands.setButtonConfig
        payload[1] = buttonIndex
        payload[3] = kindValue
        for offset, byte in enumerate(extra):
            payload[4 + offset] = byte
        return DarmosharkProtocol.longReportId, bytes(payload)

    @staticmethod
    def parseRead(reply, buttonIndex):
        if len(reply) < 5:
            raise ValueError(f"button reply too short: {len(reply)} bytes")
        if reply[0] != DmsCommands.getButtonConfig:
            raise ValueError(f"unexpected opcode 0x{reply[0]:02X} in button reply")
        if reply[1] != buttonIndex:
            raise ValueError(
                f"reply is for button {reply[1]}, expected {buttonIndex}")
        kindValue = reply[3]
        return {
            "button": buttonIndex,
            "kind": ButtonPacket.kindName(kindValue),
            "kindValue": kindValue,
            "data": bytes(reply[4:]),
        }

    @staticmethod
    def resolveKind(kind):
        if isinstance(kind, int):
            if kind not in ButtonPacket.kinds.values():
                raise ValueError(f"unknown assignment kind {kind}")
            return kind
        if kind not in ButtonPacket.kinds:
            raise ValueError(
                f"unknown assignment kind {kind!r}, expected one of "
                f"{sorted(ButtonPacket.kinds)}")
        return ButtonPacket.kinds[kind]

    @staticmethod
    def kindName(value):
        for name, candidate in ButtonPacket.kinds.items():
            if candidate == value:
                return name
        return f"unknown({value})"

    @staticmethod
    def _validateIndex(buttonIndex):
        if not 0 <= buttonIndex < ButtonPacket.buttonCount:
            raise ValueError(
                f"button must be 0-{ButtonPacket.buttonCount - 1}, "
                f"got {buttonIndex}")
