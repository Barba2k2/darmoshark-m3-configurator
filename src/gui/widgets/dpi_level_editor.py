"""One DPI level: the selectable colour button plus its value field."""

from PySide6 import QtCore, QtWidgets

from gui.theme import Theme
from gui.widgets.dpi_level_button import DpiLevelButton


class DpiLevelEditor(QtWidgets.QWidget):
    """Stacks the level button over its editable value.

    Grouping them in one widget keeps the two rows from fighting over vertical
    space in the parent layout.
    """

    selected = QtCore.Signal(int)

    def __init__(self, levelIndex, colour, value, valueRange, badge=None,
                 parent=None):
        super().__init__(parent)
        self.levelIndex = levelIndex

        layout = QtWidgets.QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(Theme.spacingSmall)

        self.button = DpiLevelButton(levelIndex, colour, str(value))
        self.button.selected.connect(self.selected.emit)

        low, high = valueRange
        self.field = QtWidgets.QSpinBox()
        self.field.setRange(low, high)
        self.field.setValue(value)
        self.field.setSingleStep(100)
        self.field.setButtonSymbols(QtWidgets.QAbstractSpinBox.NoButtons)
        self.field.setAlignment(QtCore.Qt.AlignCenter)
        self.field.setFixedWidth(Theme.levelButtonSize)
        self.field.setFixedHeight(Theme.levelFieldHeight)
        self.field.setStyleSheet(
            f"QSpinBox {{ background: {Theme.surfaceRaised};"
            f" color: {Theme.textPrimary}; border: 1px solid {Theme.border};"
            f" border-radius: {Theme.radiusSmall}px;"
            f" padding: {Theme.spacingSmall // 2}px;"
            f" font-size: {Theme.fontSizeCaption}px; }}")
        self.field.valueChanged.connect(self._syncButtonLabel)

        layout.addWidget(self.button)
        layout.addWidget(self.field)

        # The badge slot is reserved on every editor so the row keeps one
        # baseline; entries without a badge simply leave it empty and stay
        # top aligned.
        caption = QtWidgets.QLabel(badge or "")
        caption.setAlignment(QtCore.Qt.AlignCenter)
        caption.setFixedHeight(Theme.badgeHeight)
        caption.setStyleSheet(
            f"color: {colour}; font-size: {Theme.fontSizeCaption}px;"
            f" border: none; background: transparent;")
        layout.addWidget(caption)
        layout.addStretch()

        self.setFixedWidth(Theme.levelButtonSize)
        self.setFixedHeight(
            Theme.levelButtonSize + Theme.spacingSmall
            + Theme.levelFieldHeight + Theme.spacingSmall + Theme.badgeHeight)

    @property
    def value(self):
        return self.field.value()

    @property
    def isSelected(self):
        return self.button.isChecked()

    def applySelection(self, isSelected):
        self.button.setChecked(isSelected)

    def _syncButtonLabel(self, value):
        self.button.applyValueLabel(str(value))
