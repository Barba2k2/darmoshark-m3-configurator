/** All window text in one place, ready for translation. */
export class Labels {
  static readonly windowTitle = "Darmoshark M3";
  static readonly subtitleDisconnected = "mouse não encontrado";
  static readonly subtitleLoading = "procurando o mouse…";
  static readonly transportReceiver = "receptor 2.4GHz";
  static readonly transportCable = "cabo USB";
  static readonly transportOther = "interface desconhecida";

  static readonly batteryCaption = "BATERIA";
  static readonly dpiSection = "Níveis de DPI";
  static readonly dpiHint =
    "Clique num nível para ativá-lo — a cor é a mesma do LED do mouse. Qualquer campo aceita um valor livre, de 50 a 26000.";
  static readonly dpiHintCable =
    "Pelo cabo o mouse não informa os níveis gravados: os campos começam com os valores de fábrica.";
  static readonly dpiCustomBadge = "livre";
  static readonly dpiApply = "Gravar níveis";

  static readonly settingsSection = "Ajustes";
  static readonly reportRate = "Taxa de resposta";
  static readonly liftOff = "Altura de acionamento";
  static readonly debounce = "Debounce do clique";
  static readonly sleepTimer = "Suspender após";
  static readonly sleepCableOnly = "só pelo cabo";

  static readonly liftOffLow = "Baixa";
  static readonly liftOffHigh = "Alta";

  static readonly resetButton = "Voltar ao padrão de fábrica";
  static readonly resetConfirmTitle = "Restaurar padrões";
  static readonly resetConfirmBody =
    "Isto apaga todos os perfis gravados no mouse e restaura a configuração de fábrica. Continuar?";
  static readonly resetConfirm = "Restaurar";
  static readonly resetCancel = "Cancelar";

  static readonly statusReady = "Pronto";
  static readonly statusWorking = "Gravando…";
  static readonly statusApplied = "Aplicado";
  static readonly statusNoMouse = "Conecte o mouse pelo cabo USB ou pelo receptor 2.4GHz";

  static readonly unitHertz = "Hz";
  static readonly unitMilliseconds = "ms";
  static readonly unitMinutes = "min";
  static readonly unitPercent = "%";
  static readonly missingValue = "--";
}
