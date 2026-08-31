"""Selectable button representing one DPI level."""

from PySide6 import QtCore, QtWidgets

from gui.theme import Theme


class DpiLevelButton(QtWidgets.QPushButton):
    """Shows a level's indicator colour and its value.

    The colour mirrors the LED the mouse actually lights up for that level,
    so the window and the hardware agree at a glance.
    """

    selected = QtCore.Signal(int)

    def __init__(self, levelIndex, colour, valueLabel, parent=None):
        super().__init__(parent)
        self.levelIndex = levelIndex
        self.colour = colour
        self.setCheckable(True)
        self.setFixedSize(Theme.levelButtonSize, Theme.levelButtonSize)
        self.setCursor(QtCore.Qt.PointingHandCursor)
        self.setText(valueLabel)
        self._applyStyle()
        self.clicked.connect(self._emitSelected)

    def applyValueLabel(self, valueLabel):
        self.setText(valueLabel)

    def _emitSelected(self):
        self.selected.emit(self.levelIndex)

    def _applyStyle(self):
        self.setStyleSheet(
            f"QPushButton {{"
            f" background: {Theme.surfaceRaised};"
            f" color: {Theme.textSecondary};"
            f" border: 2px solid {Theme.border};"
            f" border-radius: {Theme.radius}px;"
            f" font-size: {Theme.fontSizeBody}px;"
            f" text-align: center; padding: 0px; }}"
            f"QPushButton:hover {{ border-color: {self.colour}; }}"
            f"QPushButton:checked {{"
            f" background: {self.colour}; color: #10101a;"
            f" border-color: {self.colour}; font-weight: 600; }}")
