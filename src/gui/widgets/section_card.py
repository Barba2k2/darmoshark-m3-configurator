"""Card container that groups related controls."""

from PySide6 import QtWidgets

from gui.theme import Theme


class SectionCard(QtWidgets.QFrame):
    """A titled surface. Body widgets are added by the caller."""

    def __init__(self, title, parent=None):
        super().__init__(parent)
        self.setStyleSheet(
            f"QFrame {{ background: {Theme.surface};"
            f" border: 1px solid {Theme.border};"
            f" border-radius: {Theme.radius}px; }}")

        self.body = QtWidgets.QVBoxLayout(self)
        self.body.setContentsMargins(
            Theme.spacingLarge, Theme.spacing,
            Theme.spacingLarge, Theme.spacingLarge)
        self.body.setSpacing(Theme.spacing)

        heading = QtWidgets.QLabel(title)
        heading.setStyleSheet(
            f"color: {Theme.textPrimary}; font-size: {Theme.fontSizeHeading}px;"
            f" font-weight: 600; border: none;")
        self.body.addWidget(heading)

    def addRow(self, widget):
        self.body.addWidget(widget)
