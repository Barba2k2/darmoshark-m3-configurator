"""One labelled control inside a settings panel."""

from PySide6 import QtWidgets

from gui.theme import Theme


class SettingRow(QtWidgets.QWidget):
    """Pairs a caption with an arbitrary control widget."""

    def __init__(self, caption, control, parent=None):
        super().__init__(parent)
        self.control = control

        layout = QtWidgets.QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(Theme.spacing)

        label = QtWidgets.QLabel(caption)
        label.setStyleSheet(
            f"color: {Theme.textSecondary}; font-size: {Theme.fontSizeBody}px;"
            f" border: none; background: transparent;")
        label.setMinimumWidth(140)

        layout.addWidget(label)
        layout.addStretch()
        layout.addWidget(control)
