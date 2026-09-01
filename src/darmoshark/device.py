"""HID transport for Darmoshark mice."""

import time

import hid

from darmoshark.dms_commands import DmsCommands
from darmoshark.protocol import DarmosharkProtocol


class DarmosharkDevice:
    """Opens the vendor config interface and exchanges reports with it.

    Two interfaces carry the same payloads on usage page 0x8C: the mouse
    itself over the charging cable (feature report 0x52, write only), and the
    2.4GHz receiver (feature report 0x51, reads included).
    """

    def __init__(self, handle, info):
        self.handle = handle
        self.info = info

    @staticmethod
    def discover():
        """Returns every candidate config interface, best match first."""
        candidates = []
        for entry in hid.enumerate(DarmosharkProtocol.vendorId, 0):
            usagePage = entry.get("usage_page", 0)
            if usagePage == DarmosharkProtocol.dfuUsagePage:
                candidates.insert(0, entry)
            elif usagePage >= 0xFF00:
                candidates.append(entry)
        return candidates

    @staticmethod
    def open():
        candidates = DarmosharkDevice.discover()
        if not candidates:
            raise RuntimeError(
                "no Darmoshark device found. Connect the charging cable, or plug in "
                "the 2.4GHz receiver with the mouse switch set to 2.4G."
            )

        errors = []
        for entry in candidates:
            handle = hid.device()
            try:
                handle.open_path(entry["path"])
            except (OSError, ValueError) as error:
                errors.append(f"{entry['path']}: {error}")
                continue
            return DarmosharkDevice(handle, entry)

        raise RuntimeError("could not open any interface:\n  " + "\n  ".join(errors))

    def request(self, reportId, payload, timeoutSeconds=1.0):
        """Sends a command and waits for the matching reply, or None on timeout."""
        try:
            self.handle.write(bytes([reportId]) + payload)
        except (OSError, ValueError) as error:
            raise RuntimeError(f"failed to write report 0x{reportId:02X}: {error}")

        self.handle.set_nonblocking(1)
        deadline = time.monotonic() + timeoutSeconds
        while time.monotonic() < deadline:
            reply = self.handle.read(64)
            if reply:
                return bytes(reply)
            time.sleep(0.01)
        return None

    def requestFeature(self, reportId, payload, retries=3):
        """Feature-report round trip: write the command, read the answer back."""
        if self.usesDongleTransport:
            return self.requestDongle(payload, attempts=retries)

        size = len(payload)
        for _ in range(retries):
            try:
                self.handle.send_feature_report(bytes([reportId]) + payload)
            except (OSError, ValueError) as error:
                raise RuntimeError(
                    f"failed to send feature report 0x{reportId:02X}: {error}")
            time.sleep(0.08)
            try:
                reply = bytes(self.handle.get_feature_report(reportId, size + 1))
            except (OSError, ValueError):
                reply = b""
            if len(reply) > 2 and any(reply[1:]):
                return reply
            time.sleep(0.1)
        return None

    def requestDfu(self, kind, command, timeoutSeconds=0.8):
        """Sends an AA/55 DFU frame and collects the multi-packet reply."""
        frame = bytearray(DarmosharkProtocol.dfuPayloadSize)
        frame[0] = DarmosharkProtocol.dfuHeaderByte
        frame[1] = DarmosharkProtocol.dfuSendNoAck
        frame[2] = 3
        frame[3] = (~3) & 0xFF
        frame[4] = kind
        frame[5] = command
        frame[6] = command

        try:
            self.handle.write(bytes([DarmosharkProtocol.dfuOutputId]) + bytes(frame))
        except (OSError, ValueError) as error:
            raise RuntimeError(f"failed to send DFU command {command}: {error}")

        self.handle.set_nonblocking(1)
        deadline = time.monotonic() + timeoutSeconds
        packets = []
        while time.monotonic() < deadline:
            reply = self.handle.read(64)
            if reply:
                packets.append(bytes(reply)[1:])  # drop the echoed report id
            else:
                time.sleep(0.01)
        return packets

    @property
    def usesDongleTransport(self):
        """True when the open interface belongs to the 2.4GHz receiver."""
        return self.info.get("product_id") in DarmosharkProtocol.dongleProductIds

    @property
    def usesCableTransport(self):
        """True when the open interface is the mouse's own cable one."""
        return (self.info.get("usage_page") == DarmosharkProtocol.dfuUsagePage
                and not self.usesDongleTransport)

    @staticmethod
    def dongleChannel(payload):
        """Picks the receiver report that fits the payload: 0x51 or 0x52."""
        if len(payload) <= DarmosharkProtocol.donglePayloadSize:
            return (DarmosharkProtocol.dongleConfigFeatureId,
                    DarmosharkProtocol.donglePayloadSize)
        if len(payload) <= DarmosharkProtocol.dongleLongPayloadSize:
            return (DarmosharkProtocol.dongleLongFeatureId,
                    DarmosharkProtocol.dongleLongPayloadSize)
        raise ValueError(
            f"payload of {len(payload)} bytes exceeds the "
            f"{DarmosharkProtocol.dongleLongPayloadSize}-byte receiver report")

    def sendDongle(self, payload):
        """Delivers a payload as a receiver feature report, zero padded."""
        featureId, size = DarmosharkDevice.dongleChannel(payload)
        framed = bytes(payload) + bytes(size - len(payload))
        try:
            self.handle.send_feature_report(bytes([featureId]) + framed)
        except (OSError, ValueError) as error:
            raise RuntimeError(f"failed to send receiver command: {error}")

    def requestDongle(self, payload, attempts=4, echoBytes=1):
        """Round trip over the receiver, honouring its acknowledgement states.

        The receiver answers on input report 0x54 with 0xE4 <status>: pending
        and busy both mean "ask again in a moment", ready means the reply is
        sitting in the feature report. The reply buffer keeps the previous
        answer until the new one lands, so the echoed leading bytes are what
        prove the data belongs to this request -- one byte is the opcode, and
        commands addressing a slot (a button, a macro) need two.
        """
        echo = bytes(payload[:echoBytes])
        for attempt in range(attempts):
            self.sendDongle(payload)
            status = self._awaitDongleAck()
            if status == DarmosharkProtocol.ackStatusLinkDown:
                raise RuntimeError(
                    "the receiver reports no live link to the mouse. Set the switch "
                    "to 2.4G, then unplug and replug the receiver -- a dropped link "
                    "does not recover on its own.")
            reply = self._readDongleFeature(payload)
            if reply is not None and reply[1:1 + echoBytes] == echo:
                return reply
            if attempt + 1 < attempts:
                time.sleep(0.3)
        return None

    def _awaitDongleAck(self, timeoutSeconds=0.6):
        """Returns the status byte of the 0xE4 frame, or None if none arrives."""
        self.handle.set_nonblocking(1)
        deadline = time.monotonic() + timeoutSeconds
        while time.monotonic() < deadline:
            frame = self.handle.read(64)
            if not frame:
                time.sleep(0.005)
                continue
            frame = bytes(frame)
            if len(frame) > 2 and frame[1] == DmsCommands.ackOpcode:
                return frame[2]
        return None

    def _readDongleFeature(self, payload):
        featureId, size = DarmosharkDevice.dongleChannel(payload)
        try:
            reply = bytes(self.handle.get_feature_report(featureId, size + 1))
        except (OSError, ValueError):
            return None
        return reply if len(reply) > 2 and any(reply[1:]) else None

    def sendCommand(self, reportId, payload):
        """Delivers a dms payload over whichever transport this interface uses.

        The receiver takes the payload as feature report 0x51. The cable takes
        the identical bytes as feature report 0x52, zero padded to 64 bytes.
        """
        if self.usesDongleTransport:
            return self.sendDongle(payload)
        if not self.usesCableTransport:
            return self.send(reportId, payload)

        size = DarmosharkProtocol.cableConfigFeatureSize
        if len(payload) > size:
            raise ValueError(
                f"payload of {len(payload)} bytes exceeds the {size}-byte "
                "cable config report")
        framed = bytes(payload) + bytes(size - len(payload))
        try:
            self.handle.send_feature_report(
                bytes([DarmosharkProtocol.cableConfigFeatureId]) + framed)
        except (OSError, ValueError) as error:
            raise RuntimeError(f"failed to send config command: {error}")

    def requestCommand(self, reportId, payload, timeoutSeconds=1.0):
        """Sends a dms command and returns the reply, transport aware."""
        if self.usesDongleTransport:
            return self.requestDongle(payload)
        if not self.usesCableTransport:
            return self.request(reportId, payload, timeoutSeconds)

        self.sendCommand(reportId, payload)
        time.sleep(0.08)
        try:
            reply = bytes(self.handle.get_feature_report(
                DarmosharkProtocol.cableConfigFeatureId,
                DarmosharkProtocol.cableConfigFeatureSize + 1))
        except (OSError, ValueError):
            return None
        return reply[1:] if len(reply) > 2 and any(reply[1:]) else None

    def send(self, reportId, payload):
        try:
            self.handle.write(bytes([reportId]) + payload)
        except (OSError, ValueError) as error:
            raise RuntimeError(f"failed to write report 0x{reportId:02X}: {error}")

    def close(self):
        self.handle.close()

    def __enter__(self):
        return self

    def __exit__(self, excType, excValue, traceback):
        self.close()
        return False
