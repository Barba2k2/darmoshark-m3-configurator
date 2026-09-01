"""Wire protocol constants for Darmoshark mice (contract "dms").

Reverse engineered from the official WebHID configurator bundle served by
darmoshark.cc (Angular app, Keychron launcher platform).
"""


class DarmosharkProtocol:
    """Report ids, opcodes and limits of the Darmoshark "dms" contract."""

    vendorId = 0x248A
    knownProductIds = (0xFF12, 0xFF18, 0xFF10, 0xFF30, 0xFF31)

    # Output report ids used to carry commands.
    longReportId = 0xB3          # 63-byte payload (reads, >5 dpi levels)
    shortReportId = 0xB5         # 20-byte payload (most writes)

    longPayloadSize = 63
    shortPayloadSize = 20

    # Opcodes (payload byte 0).
    cmdGetBaseInfo = 0x06
    cmdSetDpiShort = 0x40        # up to 5 levels, sent on shortReportId
    cmdSetDpiLong = 0x44         # more than 5 levels, sent on longReportId

    # Input report opcodes that carry a base-info reply.
    baseInfoReplyOpcodes = (0x05, 0x06)

    # Both transports live on usage page 0x8C: the mouse exposes it over the
    # charging cable, the receiver exposes an identical descriptor of its own.
    # The report ids 0xB3 / 0xB5 exist in the protocol but not in this
    # descriptor -- writing them reaches nothing. Confirmed on hardware.
    dfuUsagePage = 0x8C
    cableConfigFeatureId = 0x52
    cableConfigFeatureSize = 64

    # 2.4GHz receiver. It enumerates under its own product id and carries the
    # same 20-byte payloads as feature report 0x51, answering on input report
    # 0x54. Unlike the cable, it also reads configuration back.
    dongleProductIds = (0xFF30,)
    dongleConfigFeatureId = 0x51
    donglePayloadSize = 20
    # Commands that do not fit the short report take the 64-byte one, the same
    # id the cable uses -- button reads and macro data travel here.
    dongleLongFeatureId = 0x52
    dongleLongPayloadSize = 64
    dongleAckInputId = 0x54
    cmdDongleBaseInfo = 0x07     # config snapshot; the cable contract uses 0x06

    # Status byte of the 0xE4 acknowledgement the receiver posts on 0x54.
    ackStatusPending = 0         # command queued, resend until it turns ready
    ackStatusReady = 1           # reply is waiting in the feature report
    ackStatusLinkDown = 2        # receiver has no live link to the mouse
    ackStatusBusy = 4            # same as pending, receiver still working

    dpiMinimum = 50
    dpiMaximum = 26000
    maxShortLevels = 5
    maxLevels = 8

    # Cable-side channel: identify/battery only, no DPI state.
    identifyFeatureId = 0x51
    identifyPayloadSize = 20
    cmdIdentify = 0x06

    # DFU channel (cable interface, usage page 0x8C).
    dfuOutputId = 0xB2
    dfuInputId = 0xB1
    dfuPayloadSize = 32
    dfuHeaderByte = 0xAA
    dfuSendNoAck = 0x55
    dfuSendAck = 0x56
    cmdDfuModuleInfo = 96
    cmdDfuVersion = 97
