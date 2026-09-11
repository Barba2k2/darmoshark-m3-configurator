use std::thread;
use std::time::Duration;

use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::error::darmoshark_error::DarmosharkResult;
use darmoshark::packets::report_rate_packet::ReportRatePacket;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::dto::device_state_dto::DeviceStateDto;
use crate::dto::snapshot_dto::SnapshotDto;
use crate::state::device_gate::DeviceGate;
use crate::tray::menu_bar_cache::MenuBarCache;
use crate::tray::menu_bar_labels::MenuBarLabels;
use crate::tray::menu_bar_text::MenuBarText;

/// The status item in the macOS menu bar: battery and active DPI next to a
/// template icon, and a menu to switch DPI level and polling rate.
///
/// Every hardware call runs off the main thread, through the same
/// `DeviceGate` as the window, so the menu never blocks and never collides
/// with a write from the window.
pub struct MenuBar;

impl MenuBar {
  pub const trayId: &'static str = "mouse";
  /// Emitted to the webview after the menu changed a setting.
  pub const deviceChangedEvent: &'static str = "device-changed";
  pub const dpiPrefix: &'static str = "dpi:";
  pub const ratePrefix: &'static str = "rate:";
  pub const openId: &'static str = "open";
  pub const quitId: &'static str = "quit";

  /// Creates the status item and starts the battery poll.
  pub fn install(app: &AppHandle) -> tauri::Result<()> {
    app.manage(MenuBarCache::new());
    let menu = Self::build_menu(app, &Ok(None))?;
    TrayIconBuilder::with_id(Self::trayId)
      .icon(Image::from_bytes(include_bytes!(
        "../../icons/tray-template.png"
      ))?)
      .icon_as_template(true)
      .title(MenuBarLabels::missingValue)
      .menu(&menu)
      .show_menu_on_left_click(true)
      .on_menu_event(|app, event| Self::handle(app, event.id().as_ref()))
      .build(app)?;

    let handle = app.clone();
    thread::spawn(move || {
      loop {
        Self::refresh(&handle);
        thread::sleep(Duration::from_secs(30));
      }
    });
    Ok(())
  }

  /// Reads the mouse and redraws the title and the menu. Blocking: call it
  /// from a background thread, or through `refresh_later`.
  pub fn refresh(app: &AppHandle) {
    let state = app.state::<DeviceGate>().read_device_state();
    if let Ok(Some(device)) = &state {
      app.state::<MenuBarCache>().store(device.clone());
    }
    Self::draw(app, &state);
  }

  /// Redraws the title and the menu from a state already in hand.
  fn draw(app: &AppHandle, state: &DarmosharkResult<Option<DeviceStateDto>>) {
    let Some(tray) = app.tray_by_id(Self::trayId) else {
      return;
    };
    let redraw = tray
      .set_title(Some(MenuBarText::title(state)))
      .and_then(|()| Self::build_menu(app, state))
      .and_then(|menu| tray.set_menu(Some(menu)));
    if let Err(error) = redraw {
      eprintln!("menu bar: could not redraw the status item: {error}");
    }
  }

  pub fn refresh_later(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || Self::refresh(&app));
  }

  fn handle(app: &AppHandle, id: &str) {
    if id == Self::openId {
      if let Some(window) = app.get_webview_window("main") {
        let shown = window.show().and_then(|()| window.set_focus());
        if let Err(error) = shown {
          eprintln!("menu bar: could not show the configurator: {error}");
        }
      }
    } else if id == Self::quitId {
      app.exit(0);
    } else if let Some(index) = id
      .strip_prefix(Self::dpiPrefix)
      .and_then(|index| index.parse::<usize>().ok())
    {
      Self::draw_expected(app, |snapshot| snapshot.active_level = index as u8);
      Self::write_later(app, move |configurator| {
        configurator.select_dpi_level(index).map(|_| ())
      });
    } else if let Some(hertz) = id
      .strip_prefix(Self::ratePrefix)
      .and_then(|hertz| hertz.parse::<u32>().ok())
    {
      Self::draw_expected(app, |snapshot| snapshot.report_rate_hz = Some(hertz));
      Self::write_later(app, move |configurator| {
        configurator.write_report_rate(hertz)
      });
    }
  }

  /// Redraws at once with the change the click asked for, applied to the last
  /// reading. The read that follows the write replaces it with what the mouse
  /// actually holds.
  fn draw_expected(app: &AppHandle, change: impl FnOnce(&mut SnapshotDto)) {
    let Some(mut device) = app.state::<MenuBarCache>().last() else {
      return;
    };
    if let Some(snapshot) = device.snapshot.as_mut() {
      change(snapshot);
      Self::draw(app, &Ok(Some(device)));
    }
  }

  /// Runs a write in the background, then redraws the status item and tells
  /// the window to re-read the mouse.
  fn write_later(
    app: &AppHandle,
    write: impl FnOnce(&MouseConfigurator) -> DarmosharkResult<()> + Send + 'static,
  ) {
    let app = app.clone();
    thread::spawn(move || {
      let written = app.state::<DeviceGate>().with_configurator(write);
      Self::refresh(&app);
      match written {
        Ok(()) => {
          if let Err(error) = app.emit(Self::deviceChangedEvent, ()) {
            eprintln!("menu bar: could not notify the window: {error}");
          }
        }
        Err(error) => eprintln!("menu bar: write failed: {error}"),
      }
    });
  }

  fn build_menu(
    app: &AppHandle,
    state: &DarmosharkResult<Option<DeviceStateDto>>,
  ) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(
      app,
      "status",
      MenuBarText::status_line(state),
      false,
      None::<&str>,
    )?)?;

    if let Ok(Some(device)) = state
      && let Some(snapshot) = &device.snapshot
    {
      menu.append(&PredefinedMenuItem::separator(app)?)?;
      for (index, dpi) in snapshot.dpi_levels.iter().enumerate() {
        menu.append(&CheckMenuItem::with_id(
          app,
          format!("{}{index}", Self::dpiPrefix),
          format!("{dpi} {}", MenuBarLabels::unitDpi),
          true,
          index == usize::from(snapshot.active_level),
          None::<&str>,
        )?)?;
      }
      let rates = Submenu::with_id(app, "rates", MenuBarLabels::reportRate, true)?;
      for hertz in ReportRatePacket::supportedRates {
        rates.append(&CheckMenuItem::with_id(
          app,
          format!("{}{hertz}", Self::ratePrefix),
          format!("{hertz} {}", MenuBarLabels::unitHertz),
          true,
          snapshot.report_rate_hz == Some(hertz),
          None::<&str>,
        )?)?;
      }
      menu.append(&rates)?;
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
      app,
      Self::openId,
      MenuBarLabels::openConfigurator,
      true,
      None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
      app,
      Self::quitId,
      MenuBarLabels::quit,
      true,
      None::<&str>,
    )?)?;
    Ok(menu)
  }
}
