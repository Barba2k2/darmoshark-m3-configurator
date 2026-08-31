"""HID transport for Darmoshark mice."""

import time

import hid

from darmoshark.protocol import DarmosharkProtocol


class DarmosharkDevice:
    """Opens the vendor config interface and exchanges reports with it.

    The configuration channel is only reachable through the 2.4GHz dongle.
    Over the charging cable the mouse exposes just its DFU interface
    (usage page 0x8C), which silently ignores config commands.
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
            if usagePage in DarmosharkProtocol.configUsagePages:
                candidates.insert(0, entry)
            elif usagePage >= 0xFF00 or usagePage == DarmosharkProtocol.dfuUsagePage:
                candidates.append(entry)
        return candidates

    @staticmethod
    def open():
        candidates = DarmosharkDevice.discover()
        if not candidates:
            raise RuntimeError(
                "no Darmoshark device found. Plug in the 2.4GHz dongle and set the "
                "mouse switch to 2.4G — the charging cable only exposes DFU."
            )

        errors = []
        for entry in candidates:
            handle = hid.device()
            try:
                handle.open_path(entry["path"])
            except Exception as error:
                errors.append(f"{entry['path']}: {error}")
                continue
            return DarmosharkDevice(handle, entry)

        raise RuntimeError("could not open any interface:\n  " + "\n  ".join(errors))

    def request(self, reportId, payload, timeoutSeconds=1.0):
        """Sends a command and waits for the matching reply, or None on timeout."""
        try:
            self.handle.write(bytes([reportId]) + payload)
        except Exception as error:
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
        size = len(payload)
        for _ in range(retries):
            try:
                self.handle.send_feature_report(bytes([reportId]) + payload)
            except Exception as error:
                raise RuntimeError(
                    f"failed to send feature report 0x{reportId:02X}: {error}")
            time.sleep(0.08)
            try:
                reply = bytes(self.handle.get_feature_report(reportId, size + 1))
            except Exception:
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
        except Exception as error:
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
    def usesCableTransport(self):
        """True when the open interface is the cable one (usage page 0x8C)."""
        return self.info.get("usage_page") == DarmosharkProtocol.dfuUsagePage

    def sendCommand(self, reportId, payload):
        """Delivers a dms payload over whichever transport this interface uses.

        The dongle takes the payload on its own output report. The cable takes
        the identical bytes as feature report 0x52, zero padded to 64 bytes.
        """
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
        except Exception as error:
            raise RuntimeError(f"failed to send config command: {error}")

    def requestCommand(self, reportId, payload, timeoutSeconds=1.0):
        """Sends a dms command and returns the reply, transport aware."""
        if not self.usesCableTransport:
            return self.request(reportId, payload, timeoutSeconds)

        self.sendCommand(reportId, payload)
        time.sleep(0.08)
        try:
            reply = bytes(self.handle.get_feature_report(
                DarmosharkProtocol.cableConfigFeatureId,
                DarmosharkProtocol.cableConfigFeatureSize + 1))
        except Exception:
            return None
        return reply[1:] if len(reply) > 2 and any(reply[1:]) else None

    def send(self, reportId, payload):
        try:
            self.handle.write(bytes([reportId]) + payload)
        except Exception as error:
            raise RuntimeError(f"failed to write report 0x{reportId:02X}: {error}")

    def close(self):
        self.handle.close()

    def __enter__(self):
        return self

    def __exit__(self, excType, excValue, traceback):
        self.close()
        return False
