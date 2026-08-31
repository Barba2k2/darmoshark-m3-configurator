"""Configurator window."""

from PySide6 import QtCore, QtWidgets

from gui.labels import Labels
from gui.theme import Theme
from gui.widgets.battery_gauge import BatteryGauge
from gui.widgets.device_header import DeviceHeader
from gui.widgets.dpi_level_editor import DpiLevelEditor
from gui.widgets.section_card import SectionCard
from gui.widgets.setting_row import SettingRow


class MainWindow(QtWidgets.QMainWindow):
    """Wires the widgets to the mouse service."""

    batteryPollInterval = 30000

    def __init__(self, service):
        super().__init__()
        self.service = service
        self.levelEditors = []

        self.setWindowTitle(Labels.windowTitle)
        self.setFixedSize(Theme.windowWidth, Theme.windowHeight)
        self.setStyleSheet(f"background: {Theme.background};")

        root = QtWidgets.QWidget()
        self.setCentralWidget(root)
        layout = QtWidgets.QVBoxLayout(root)
        layout.setContentsMargins(
            Theme.spacingSection, Theme.spacingLarge,
            Theme.spacingSection, Theme.spacingLarge)
        layout.setSpacing(Theme.spacingLarge)

        self.header = DeviceHeader(Labels.windowTitle, Labels.statusReady)
        layout.addWidget(self.header)

        self.battery = BatteryGauge(Labels.batteryCaption)
        layout.addWidget(self.battery)

        layout.addWidget(self._buildDpiCard())
        layout.addWidget(self._buildSettingsCard())
        layout.addStretch()
        layout.addWidget(self._buildResetButton())

        self.status = self.statusBar()
        self.status.setStyleSheet(
            f"color: {Theme.textSecondary}; font-size: {Theme.fontSizeCaption}px;")
        self.status.showMessage(Labels.statusReady)

        self.timer = QtCore.QTimer(self)
        self.timer.timeout.connect(self._refreshIdentity)
        self.timer.start(MainWindow.batteryPollInterval)
        self._refreshIdentity()

    def _buildDpiCard(self):
        card = SectionCard(Labels.dpiSection)

        hint = QtWidgets.QLabel(Labels.dpiHint)
        hint.setWordWrap(True)
        hint.setStyleSheet(
            f"color: {Theme.textSecondary}; font-size: {Theme.fontSizeCaption}px;"
            f" border: none;")
        card.addRow(hint)

        levelRow = QtWidgets.QWidget()
        levelRow.setFixedHeight(
            Theme.levelButtonSize + Theme.spacingSmall
            + Theme.levelFieldHeight + Theme.spacingSmall + Theme.badgeHeight)
        levelLayout = QtWidgets.QHBoxLayout(levelRow)
        levelLayout.setContentsMargins(0, 0, 0, 0)
        levelLayout.setSpacing(Theme.spacingSmall)

        profile = self.service.profile
        for index, value in enumerate(profile.dpiLevels):
            colour = profile.dpiColors[index] if index < len(profile.dpiColors) \
                else Theme.accent
            # The last slot carries the badge: it is the one meant for
            # experimenting with an arbitrary value. Only five levels fit --
            # the cable transport rejects the extended (6+) packet format.
            isLast = index == len(profile.dpiLevels) - 1
            editor = DpiLevelEditor(
                index, colour, value, profile.dpiRange,
                badge=Labels.dpiCustomBadge if isLast else None)
            editor.selected.connect(self._selectLevel)
            levelLayout.addWidget(editor)
            self.levelEditors.append(editor)

        card.addRow(levelRow)

        apply = QtWidgets.QPushButton(Labels.dpiApply)
        apply.setCursor(QtCore.Qt.PointingHandCursor)
        apply.setStyleSheet(self._primaryButtonStyle())
        apply.clicked.connect(self._applyLevels)
        card.addRow(apply)
        return card

    def _buildSettingsCard(self):
        card = SectionCard(Labels.settingsSection)

        self.rateBox = QtWidgets.QComboBox()
        for hertz, _ in self.service.profile.reportRates:
            self.rateBox.addItem(f"{hertz} {Labels.unitHertz}", hertz)
        self.rateBox.setCurrentIndex(self.rateBox.count() - 1)
        self.rateBox.setStyleSheet(self._inputStyle())
        self.rateBox.activated.connect(self._applyReportRate)
        card.addRow(SettingRow(Labels.reportRate, self.rateBox))

        self.liftBox = QtWidgets.QComboBox()
        captions = {1: Labels.liftOffLow, 2: Labels.liftOffHigh}
        for index, _ in self.service.profile.liftOffSteps:
            self.liftBox.addItem(captions.get(index, str(index)), index)
        self.liftBox.setStyleSheet(self._inputStyle())
        self.liftBox.activated.connect(self._applyLiftOff)
        card.addRow(SettingRow(Labels.liftOff, self.liftBox))

        self.debounceBox = QtWidgets.QSpinBox()
        self.debounceBox.setRange(0, 20)
        self.debounceBox.setValue(8)
        self.debounceBox.setSuffix(f" {Labels.unitMilliseconds}")
        self.debounceBox.setStyleSheet(self._inputStyle())
        self.debounceBox.editingFinished.connect(self._applyDebounce)
        card.addRow(SettingRow(Labels.debounce, self.debounceBox))

        self.sleepBox = QtWidgets.QSpinBox()
        self.sleepBox.setRange(0, 255)
        self.sleepBox.setValue(10)
        self.sleepBox.setSuffix(f" {Labels.unitMinutes}")
        self.sleepBox.setStyleSheet(self._inputStyle())
        self.sleepBox.editingFinished.connect(self._applySleep)
        card.addRow(SettingRow(Labels.sleepTimer, self.sleepBox))
        return card

    def _buildResetButton(self):
        button = QtWidgets.QPushButton(Labels.resetButton)
        button.setCursor(QtCore.Qt.PointingHandCursor)
        button.setStyleSheet(
            f"QPushButton {{ background: transparent; color: {Theme.danger};"
            f" border: 1px solid {Theme.danger};"
            f" border-radius: {Theme.radiusSmall}px;"
            f" padding: {Theme.spacing}px; font-size: {Theme.fontSizeButton}px; }}"
            f"QPushButton:hover {{ background: {Theme.danger}; color: #10101a; }}")
        button.clicked.connect(self._confirmReset)
        return button

    def _selectLevel(self, levelIndex):
        for editor in self.levelEditors:
            editor.applySelection(editor.levelIndex == levelIndex)
        self._runGuarded(
            lambda: self.service.applyDpiLevels(self._currentValues(), levelIndex))

    def _applyLevels(self):
        active = self._activeLevel()
        self._runGuarded(
            lambda: self.service.applyDpiLevels(self._currentValues(), active))

    def _applyReportRate(self):
        hertz = self.rateBox.currentData()
        self._runGuarded(
            lambda: self.service.applyReportRate(
                hertz, len(self.levelEditors), self._activeLevel()))

    def _applyLiftOff(self):
        value = self.liftBox.currentData()
        self._runGuarded(lambda: self.service.applyLiftOff(value))

    def _applyDebounce(self):
        value = self.debounceBox.value()
        self._runGuarded(lambda: self.service.applyDebounce(value))

    def _applySleep(self):
        value = self.sleepBox.value()
        self._runGuarded(lambda: self.service.applySleepTimer(value))

    def _confirmReset(self):
        answer = QtWidgets.QMessageBox.question(
            self, Labels.resetConfirmTitle, Labels.resetConfirmBody)
        if answer == QtWidgets.QMessageBox.Yes:
            self._runGuarded(self.service.restoreDefaults)

    def _currentValues(self):
        return [editor.value for editor in self.levelEditors]

    def _activeLevel(self):
        return next((e.levelIndex for e in self.levelEditors if e.isSelected), 0)

    def _refreshIdentity(self):
        if not self.service.isConnected:
            self.header.applySubtitle(Labels.subtitleDisconnected)
            self.status.showMessage(Labels.statusNoMouse)
            return
        try:
            info = self.service.readIdentity()
        except (RuntimeError, ValueError):
            self.header.applySubtitle(Labels.subtitleDisconnected)
            return
        self.header.applySubtitle(f"firmware {info.firmwareVersion}")
        self.battery.applyReading(
            info.batteryPercent, f"{info.batteryPercent}{Labels.unitPercent}")

    def _runGuarded(self, action):
        try:
            action()
        except (RuntimeError, ValueError) as error:
            self.status.showMessage(str(error))
            return
        self.status.showMessage(Labels.statusApplied)
        self._refreshIdentity()

    def _primaryButtonStyle(self):
        return (f"QPushButton {{ background: {Theme.accentStrong};"
                f" color: {Theme.textPrimary};"
                f" border: none; border-radius: {Theme.radiusSmall}px;"
                f" padding: {Theme.spacing}px; font-size: {Theme.fontSizeButton}px;"
                f" font-weight: 600; }}"
                f"QPushButton:hover {{ background: {Theme.accentHover}; }}")

    def _inputStyle(self):
        return (f"QComboBox, QSpinBox {{ background: {Theme.surfaceRaised};"
                f" color: {Theme.textPrimary}; border: 1px solid {Theme.border};"
                f" border-radius: {Theme.radiusSmall}px;"
                f" padding: {Theme.spacingSmall}px;"
                f" font-size: {Theme.fontSizeBody}px; min-width: 120px; }}"
                f"QComboBox::drop-down {{ border: none; }}"
                f"QComboBox QAbstractItemView {{ background: {Theme.surfaceRaised};"
                f" color: {Theme.textPrimary};"
                f" selection-background-color: {Theme.accent}; }}")
