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

    # The config channel is reachable two ways. Over the 2.4GHz dongle the
    # commands ride raw output reports (0xB3 / 0xB5). Over the charging cable
    # the very same payloads are accepted as feature report 0x52 on the
    # usage page 0x8C interface -- confirmed on hardware.
    dfuUsagePage = 0x8C
    configUsagePages = (0xFF0A, 0xFFC1)
    cableConfigFeatureId = 0x52
    cableConfigFeatureSize = 64

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
