"""Header showing which device is connected."""

from PySide6 import QtWidgets

from gui.theme import Theme


class DeviceHeader(QtWidgets.QWidget):
    """Device name plus a secondary line. Text comes from the caller."""

    def __init__(self, title, subtitle, parent=None):
        super().__init__(parent)
        layout = QtWidgets.QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(Theme.spacingSmall // 2)

        self.title = QtWidgets.QLabel(title)
        self.title.setStyleSheet(
            f"color: {Theme.textPrimary}; font-size: {Theme.fontSizeTitle}px;"
            f" font-weight: 600;")

        self.subtitle = QtWidgets.QLabel(subtitle)
        self.subtitle.setStyleSheet(
            f"color: {Theme.textSecondary}; font-size: {Theme.fontSizeBody}px;")

        layout.addWidget(self.title)
        layout.addWidget(self.subtitle)

    def applySubtitle(self, text):
        self.subtitle.setText(text)
