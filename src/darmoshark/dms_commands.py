"""Opcode table of the Darmoshark "dms" configuration protocol.

Names mirror the CMD_* symbols found in the official configurator bundle.
Commands travel as raw payloads on two output reports: the 20-byte short
report (0xB5) and the 63-byte long report (0xB3).
"""


class DmsCommands:
    """Opcodes, report routing and reply markers."""

    # Base / device
    getProtocol = 2
    getBondInfo = 3
    getDeviceString = 4
    getMouseInfo = 5
    getMouseExtInfo = 6          # the "base info" snapshot
    deviceTime = 10              # sleep timer get/set
    pairButton = 11
    profileSwitch = 14
    driverConfigRecovery = 15    # factory reset when value = 63

    # Lighting
    getLightEffectParam = 35
    setLightEffectParam = 36

    # Mouse tuning
    setDpi = 64
    setReportRate = 65
    setSensorLiftCutoff = 66     # LOD and assorted sensor toggles
    setButtonDebounce = 67
    setDpiExtended = 68          # more than 5 levels
    setScroll = 69

    # Buttons and macros
    setButtonConfig = 82
    setMacroName = 83
    setMacroData = 84
    getAllButtonConfig = 97
    getButtonConfig = 98
    getMacroName = 99
    getMacroData = 100

    # Long data transfer
    longDataTransfer = 113
    longDataFlowControl = 114

    # Reply markers
    ackOpcode = 0xE4             # [0]=0xE4 [1]=status [2]=echoed opcode
    ackStatusOk = 0
    lightChangedEvent = 225
    baseChangedEvent = 226
    profileChangedEvent = 229

    factoryResetValue = 63

    # Report routing: opcodes that must go on the long (0xB3) report.
    longReportCommands = frozenset({
        getMouseExtInfo, getDeviceString, setDpiExtended, setButtonConfig,
        setMacroName, setMacroData, getAllButtonConfig, getButtonConfig,
        getMacroName, getMacroData, longDataTransfer,
    })

    @staticmethod
    def usesLongReport(opcode):
        return opcode in DmsCommands.longReportCommands
