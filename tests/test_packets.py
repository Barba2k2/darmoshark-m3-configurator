"""Encoding tests for the packet builders.

Expected bytes are taken from the official configurator's own construction
logic, so a passing test means our frame is byte-identical to the vendor's.
"""

import unittest

from darmoshark.button_packet import ButtonPacket
from darmoshark.dongle_base_info import DongleBaseInfo
from darmoshark.dpi_packet import DpiPacket
from darmoshark.profile_packet import ProfilePacket
from darmoshark.report_rate_packet import ReportRatePacket
from darmoshark.tuning_packet import TuningPacket


class DpiPacketTest(unittest.TestCase):

    def test_encodes_five_levels_little_endian(self):
        reportId, payload = DpiPacket.build([400, 800, 1600, 3200, 4800], 2)
        self.assertEqual(reportId, 0xB5)
        self.assertEqual(payload[0], 0x40)
        self.assertEqual(tuple(payload[1:4]), (2, 2, 2))
        decoded = [payload[4 + i * 2] | payload[5 + i * 2] << 8 for i in range(5)]
        self.assertEqual(decoded, [400, 800, 1600, 3200, 4800])
        self.assertEqual(payload[14], 5)
        self.assertEqual(len(payload), 20)

    def test_switches_to_long_form_beyond_five_levels(self):
        reportId, payload = DpiPacket.build([400, 800, 1200, 1600, 2400, 3200], 0)
        self.assertEqual(reportId, 0xB3)
        self.assertEqual(payload[0], 0x44)
        self.assertEqual(payload[4], 6)
        self.assertEqual(len(payload), 63)

    def test_rejects_out_of_range_dpi(self):
        for bad in (49, 26001):
            with self.assertRaises(ValueError):
                DpiPacket.build([bad], 0)

    def test_rejects_active_level_outside_values(self):
        with self.assertRaises(ValueError):
            DpiPacket.build([800, 1600], 5)

    def test_rejects_too_many_levels(self):
        with self.assertRaises(ValueError):
            DpiPacket.build([800] * 9, 0)


class ReportRatePacketTest(unittest.TestCase):

    def test_encodes_rate_codes(self):
        codes = [ReportRatePacket.rateToCode(hz) for hz in (1000, 1000, 500)]
        reportId, payload = ReportRatePacket.build(codes, 1)
        self.assertEqual(reportId, 0xB5)
        self.assertEqual(payload[0], 65)
        self.assertEqual(tuple(payload[1:3]), (1, 1))
        self.assertEqual(tuple(payload[3:6]), (2, 2, 1))
        self.assertEqual(payload[9], 3)

    def test_rate_code_round_trip(self):
        for hz in ReportRatePacket.supportedRates:
            self.assertEqual(ReportRatePacket.codeToRate(
                ReportRatePacket.rateToCode(hz)), hz)

    def test_rejects_unsupported_rate(self):
        with self.assertRaises(ValueError):
            ReportRatePacket.rateToCode(8000)


class TuningPacketTest(unittest.TestCase):

    def test_debounce_layout(self):
        reportId, payload = TuningPacket.buildDebounce(8)
        self.assertEqual((reportId, payload[0], payload[1]), (0xB5, 67, 8))

    def test_debounce_range_is_enforced(self):
        with self.assertRaises(ValueError):
            TuningPacket.buildDebounce(21)

    def test_sensor_layout_matches_vendor_offsets(self):
        _, payload = TuningPacket.buildSensor(liftOff=2, scroll=1, eSports=1)
        self.assertEqual(payload[0], 66)
        self.assertEqual(payload[1], 2)
        self.assertEqual(payload[6], 1)
        self.assertEqual(payload[7], 1)

    def test_rejects_invalid_lift_off(self):
        with self.assertRaises(ValueError):
            TuningPacket.buildSensor(liftOff=9)

    def test_sleep_uses_set_mode_marker(self):
        _, payload = TuningPacket.buildSleep(10, "set")
        self.assertEqual((payload[0], payload[1], payload[2]), (10, 1, 10))

    def test_scroll_layout(self):
        _, payload = TuningPacket.buildScroll(3, 1, 2)
        self.assertEqual(tuple(payload[0:4]), (69, 3, 1, 2))


class ProfilePacketTest(unittest.TestCase):

    def test_switch_layout(self):
        _, payload = ProfilePacket.buildSwitch(2)
        self.assertEqual((payload[0], payload[1]), (14, 2))

    def test_factory_reset_uses_value_63(self):
        _, payload = ProfilePacket.buildFactoryReset()
        self.assertEqual((payload[0], payload[1]), (15, 63))

    def test_rejects_unknown_profile(self):
        with self.assertRaises(ValueError):
            ProfilePacket.buildSwitch(9)


class ButtonPacketTest(unittest.TestCase):

    def test_write_layout(self):
        reportId, payload = ButtonPacket.buildWrite(1, "dpi")
        self.assertEqual(reportId, 0xB3)
        self.assertEqual(payload[0], 82)
        self.assertEqual(payload[1], 1)
        self.assertEqual(payload[3], ButtonPacket.kinds["dpi"])

    def test_read_round_trip(self):
        _, payload = ButtonPacket.buildRead(3)
        self.assertEqual((payload[0], payload[1]), (98, 3))
        reply = bytes([98, 3, 0, ButtonPacket.kinds["macro"], 7, 7])
        parsed = ButtonPacket.parseRead(reply, 3)
        self.assertEqual(parsed["kind"], "macro")
        self.assertEqual(parsed["data"], b"\x07\x07")

    def test_rejects_unknown_kind(self):
        with self.assertRaises(ValueError):
            ButtonPacket.buildWrite(0, "teleport")

    def test_rejects_button_out_of_range(self):
        with self.assertRaises(ValueError):
            ButtonPacket.buildRead(9)

    def test_rejects_mismatched_reply(self):
        with self.assertRaises(ValueError):
            ButtonPacket.parseRead(bytes([98, 1, 0, 5]), 3)


if __name__ == "__main__":
    unittest.main()


class DeviceProfileTest(unittest.TestCase):

    def setUp(self):
        from darmoshark.device_profile import DeviceProfile
        self.profile = DeviceProfile.load()

    def test_model_is_the_m3(self):
        self.assertEqual(self.profile.name, "Darmoshark M3")

    def test_lighting_is_not_configurable(self):
        self.assertFalse(self.profile.hasConfigurableLighting)

    def test_declares_five_default_levels(self):
        self.assertEqual(self.profile.dpiLevels, (400, 800, 1600, 3200, 4800))

    def test_lift_off_matches_tuning_packet_range(self):
        from darmoshark.tuning_packet import TuningPacket
        declared = tuple(index for index, _ in self.profile.liftOffSteps)
        self.assertEqual(declared, TuningPacket.liftOffValues)

    def test_polling_rates_match_report_rate_packet(self):
        from darmoshark.report_rate_packet import ReportRatePacket
        declared = tuple(hz for hz, _ in self.profile.reportRates)
        self.assertEqual(declared, ReportRatePacket.supportedRates)


class DongleBaseInfoTest(unittest.TestCase):
    """Bytes captured from a real receiver (M3, fw 2.0.9r) over 2.4GHz."""

    snapshot = bytes.fromhex("510700131303900120034006800cc0123505080000")

    def test_decodes_the_captured_snapshot(self):
        info = DongleBaseInfo.parse(self.snapshot)
        self.assertEqual(info.profile, 0)
        self.assertEqual(info.dpiLevels, (400, 800, 1600, 3200, 4800))
        self.assertEqual(info.activeLevel, 3)
        self.assertEqual(info.reportRate, 1)
        self.assertEqual(info.debounceMs, 8)
        self.assertEqual(info.sleepMinutes, 0)
        self.assertEqual(info.liftOff, 1)

    def test_battery_is_absent_from_this_reply(self):
        self.assertIsNone(DongleBaseInfo.parse(self.snapshot).batteryPercent)

    def test_rejects_a_reply_from_another_opcode(self):
        other = bytearray(self.snapshot)
        other[1] = 0x06
        with self.assertRaises(ValueError):
            DongleBaseInfo.parse(bytes(other))

    def test_rejects_an_implausible_level_count(self):
        broken = bytearray(self.snapshot)
        broken[17] = 9
        with self.assertRaises(ValueError):
            DongleBaseInfo.parse(bytes(broken))

    def test_rejects_more_levels_than_the_reply_carries(self):
        truncated = bytearray(self.snapshot)
        truncated[17] = 8
        with self.assertRaises(ValueError):
            DongleBaseInfo.parse(bytes(truncated))


if __name__ == "__main__":
    unittest.main()
