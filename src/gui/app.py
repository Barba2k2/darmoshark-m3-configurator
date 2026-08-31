"""Entry point for the graphical configurator."""

import sys

from PySide6 import QtWidgets

from gui.main_window import MainWindow
from gui.mouse_service import MouseService


class ConfiguratorApp:
    """Boots Qt and shows the window."""

    @staticmethod
    def launch(argv=None):
        application = QtWidgets.QApplication(argv or sys.argv)
        application.setApplicationName("Darmoshark M3")
        window = MainWindow(MouseService())
        window.show()
        return application.exec()


if __name__ == "__main__":
    sys.exit(ConfiguratorApp.launch())
