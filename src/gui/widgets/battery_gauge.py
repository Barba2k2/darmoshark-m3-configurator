"""Battery level bar."""

from PySide6 import QtWidgets

from gui.theme import Theme


class BatteryGauge(QtWidgets.QWidget):
    """A labelled bar showing charge. All copy is supplied by the caller."""

    def __init__(self, caption, parent=None):
        super().__init__(parent)
        layout = QtWidgets.QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(Theme.spacingSmall)

        self.caption = QtWidgets.QLabel(caption)
        self.caption.setStyleSheet(
            f"color: {Theme.textSecondary}; font-size: {Theme.fontSizeCaption}px;")

        self.bar = QtWidgets.QProgressBar()
        self.bar.setRange(0, 100)
        self.bar.setTextVisible(False)
        self.bar.setFixedHeight(Theme.gaugeHeight)
        self.bar.setStyleSheet(
            f"QProgressBar {{ background: {Theme.surfaceRaised};"
            f" border: none; border-radius: {Theme.radiusSmall // 2}px; }}"
            f"QProgressBar::chunk {{ background: {Theme.success};"
            f" border-radius: {Theme.radiusSmall // 2}px; }}")

        self.value = QtWidgets.QLabel("--")
        self.value.setStyleSheet(
            f"color: {Theme.textPrimary}; font-size: {Theme.fontSizeBody}px;")

        header = QtWidgets.QHBoxLayout()
        header.setContentsMargins(0, 0, 0, 0)
        header.addWidget(self.caption)
        header.addStretch()
        header.addWidget(self.value)

        layout.addLayout(header)
        layout.addWidget(self.bar)

    def applyReading(self, percent, label):
        self.bar.setValue(max(0, min(100, percent)))
        self.value.setText(label)
        colour = Theme.success if percent > 20 else Theme.danger
        self.bar.setStyleSheet(
            f"QProgressBar {{ background: {Theme.surfaceRaised};"
            f" border: none; border-radius: {Theme.radiusSmall // 2}px; }}"
            f"QProgressBar::chunk {{ background: {colour};"
            f" border-radius: {Theme.radiusSmall // 2}px; }}")
