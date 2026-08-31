"""Builders for profile switching and configuration recovery."""

from darmoshark.dms_commands import DmsCommands
from darmoshark.protocol import DarmosharkProtocol


class ProfilePacket:
    """Profile selection and the factory-reset path.

    Recovery is the same opcode for both: a partial reset targets one profile,
    while value 63 wipes every stored profile.
    """

    profileCount = 4

    @staticmethod
    def buildSwitch(profileIndex):
        if not 0 <= profileIndex < ProfilePacket.profileCount:
            raise ValueError(
                f"profile must be 0-{ProfilePacket.profileCount - 1}, "
                f"got {profileIndex}")
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.profileSwitch
        payload[1] = profileIndex
        return DarmosharkProtocol.shortReportId, bytes(payload)

    @staticmethod
    def buildRecovery(value=0, profileIndex=0, tagValue=0):
        payload = bytearray(DarmosharkProtocol.shortPayloadSize)
        payload[0] = DmsCommands.driverConfigRecovery
        payload[1] = value
        payload[2] = profileIndex
        payload[3] = tagValue
        return DarmosharkProtocol.shortReportId, bytes(payload)

    @staticmethod
    def buildFactoryReset():
        return ProfilePacket.buildRecovery(value=DmsCommands.factoryResetValue)
