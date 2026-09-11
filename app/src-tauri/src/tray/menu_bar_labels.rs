/// Every string the menu bar shows (Portuguese), in one place.
pub struct MenuBarLabels;

impl MenuBarLabels {
  pub const missingValue: &'static str = "--";
  pub const batteryPrefix: &'static str = "Bateria";
  pub const transportReceiver: &'static str = "receptor 2.4GHz";
  pub const transportCable: &'static str = "cabo USB";
  pub const transportOther: &'static str = "interface desconhecida";
  pub const noMouse: &'static str = "Mouse não encontrado";
  pub const wakeMouse: &'static str = "Mexa o mouse para acordá-lo";
  pub const reportRate: &'static str = "Taxa de resposta";
  pub const unitDpi: &'static str = "DPI";
  pub const unitHertz: &'static str = "Hz";
  pub const openConfigurator: &'static str = "Abrir configurador";
  pub const quit: &'static str = "Sair";
}
