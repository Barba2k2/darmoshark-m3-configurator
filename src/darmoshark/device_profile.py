"""Device capability profile, as published by the vendor for this model."""

import json
import pathlib


class DeviceProfile:
    """Wraps the official v3.json definition of the mouse.

    It declares what the hardware can actually do -- DPI range, selectable
    polling rates, lift-off steps and whether the model has controllable
    lighting. The M3 ships with "light": null, meaning no configurable RGB.
    """

    profilePath = pathlib.Path(__file__).with_name("m3_profile.json")

    def __init__(self, data):
        self.data = data

    @staticmethod
    def load(path=None):
        target = pathlib.Path(path) if path else DeviceProfile.profilePath
        if not target.exists():
            raise RuntimeError(f"device profile not found at {target}")
        try:
            return DeviceProfile(json.loads(target.read_text(encoding="utf-8")))
        except json.JSONDecodeError as error:
            raise RuntimeError(f"device profile is not valid JSON: {error}")

    @property
    def name(self):
        return self.data.get("name", "unknown")

    @property
    def hasConfigurableLighting(self):
        """True only when the vendor declares a light block for this model."""
        return bool(self.data.get("light"))

    @property
    def dpiLevels(self):
        return tuple(self.data.get("dpi", {}).get("level", ()))

    @property
    def dpiRange(self):
        limit = self.data.get("dpi", {}).get("limit", [])
        return (limit[0], limit[1]) if len(limit) == 2 else (None, None)

    @property
    def dpiColors(self):
        """Indicator colour per DPI level, in level order."""
        return tuple(self.data.get("dpi", {}).get("colors", ()))

    @property
    def reportRates(self):
        """(hertz, colour) for every selectable polling rate."""
        return tuple(
            (entry.get("value"), entry.get("color"))
            for entry in self.data.get("dpi", {}).get("reportRate", ())
        )

    @property
    def liftOffSteps(self):
        return tuple(
            (entry.get("index"), entry.get("value"))
            for entry in self.data.get("sys", {}).get("lod", ())
        )

    @property
    def buttonCount(self):
        return len(self.data.get("keys", ()))
