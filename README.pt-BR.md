# Darmoshark M3 — configurador aberto

[English](README.md) · **Português**

Controle de DPI, taxa de resposta e demais ajustes do mouse **Darmoshark M3**
(vendido também como **Attack Shark M3**) no **macOS**, sem o software oficial
e sem precisar do receptor 2.4GHz — funciona pelo cabo USB-C.

O protocolo foi obtido por engenharia reversa do configurador WebHID oficial
(`darmoshark.cc`) e validado no hardware. Não havia documentação pública desse
protocolo em lugar nenhum.

## Por que existe

O software do fabricante é uma página WebHID que depende de servidores na
China, e não há aplicativo nativo para macOS. Este projeto entrega os mesmos
ajustes em uma CLI e uma janela nativa, funcionando offline.

## Instalação

Só precisa de Rust, Node e pnpm.

```bash
cargo build --release -p darmoshark-cli
```

```bash
cd app && pnpm install && pnpm tauri build
```

O segundo gera `target/release/bundle/macos/Darmoshark M3.app`.

## Uso

### Interface gráfica

```bash
cd app && pnpm tauri dev
```

Cinco níveis de DPI, cada um com a cor real do LED indicador do mouse, campos
livres de 50 a 26000, taxa de resposta, altura de acionamento, debounce e
temporizador de suspensão. Pelo receptor a janela abre com os valores gravados
no mouse; pelo cabo, que não consegue lê-los, com os de fábrica.

O app vive na barra de menus, não no Dock: o item mostra bateria e DPI ativo
(`25% · 3200`), e o menu troca o nível de DPI e a taxa de resposta ou abre a
janela. Fechar a janela mantém o app na barra; "Sair" encerra.
`--` quer dizer sem leitura — em geral o mouse dormiu; mexer nele o acorda.

### Linha de comando

```bash
target/release/dms <comando>
```

| Comando | Descrição | Cabo |
|---|---|---|
| `list` | interfaces HID expostas pelo mouse | ✅ |
| `capabilities` | o que o modelo suporta (offline) | ✅ |
| `colors` | legenda de cores do LED | ✅ |
| `battery` | identidade e bateria | ✅ |
| `dfu` | módulo, firmware e revisão de hardware | ✅ |
| `dpi 400 800 1600 3200 4800 --active 3` | grava os níveis | ✅ |
| `use 3` | troca o nível ativo | ✅ |
| `rate 1000` | taxa de resposta: 125, 500 ou 1000 Hz | ❔ |
| `debounce 8` | debounce do clique, em ms | ✅ |
| `lod 1` | altura de acionamento (1 baixa, 2 alta) | ✅ |
| `sleep 10` | suspender após N minutos | ✅ |
| `profile 0` | troca o perfil interno | ✅ |
| `button 3 dpi` | remapeia um botão | ✅ |
| `reset` | restaura o padrão de fábrica | ✅ |
| `info` | configuração completa gravada | ⚠️ só com receptor |
| `buttons` | atribuições atuais dos botões | ⚠️ só com receptor |
| `bond` | com qual mouse o receptor está pareado | ⚠️ só com receptor |

Descobrir o DPI ativo sem o receptor é possível pela cor do LED — `colors`
mostra a legenda.

## Cabo ou receptor

Os dois estão validados em hardware. O receptor é o que consegue ler de volta a
configuração gravada no mouse.

| Operação | Cabo (feature `0x52`) | Receptor 2.4GHz (feature `0x51`) |
|---|---|---|
| Escrever configuração | ✅ | ✅ |
| Ler configuração | ❌ devolve sempre a identidade | ✅ |
| Identidade, firmware, bateria | ✅ | ✅ |
| Até 5 níveis de DPI | ✅ | ✅ |
| 6 ou mais níveis | ❌ formato estendido é ignorado | ❌ formato longo não tem rota |
| Taxa de resposta | ❔ sem leitura de volta | ✅ |
| Timer de suspensão | ✅ escrita (sem leitura de volta) | ❌ não é repassado |
| Leitura do bootloader | a do mouse | a do **próprio receptor** |

O receptor perde o link de configuração em silêncio: o cursor continua
funcionando enquanto todo comando responde "sem link". Tirar e recolocar o
receptor é a única recuperação.

Detalhes completos do protocolo em [PROTOCOL.pt-BR.md](PROTOCOL.pt-BR.md): os três canais
HID, a tabela de 31 opcodes, o formato de cada pacote e os canais que **não**
funcionam.

## Hardware validado

| | |
|---|---|
| Modelo | Darmoshark M3 (Attack Shark M3) |
| VID:PID | `0x248A:0xFF12` |
| Módulo | `MOTO_M3` |
| Firmware | `2.0.9r` |
| Hardware | `1.0.0` |
| Sensor | PAW3395, 50–26000 DPI |
| Bateria | 500 mAh |
| Receptor | `0x248A:0xFF30`, módulo `UCFRF001`, fw `e.1.0r-7` |
| Sistema | macOS 26.6 (arm64) |

Outros modelos Darmoshark que usam o mesmo protocolo podem funcionar, mas não
foram testados.

## Estrutura

```
crates/darmoshark/ protocolo, montagem de pacotes, decodificadores, transporte HID (Rust)
crates/cli/        `dms`, a interface de linha de comando
app/               janela Tauri: React + Zustand em src/, comandos em src-tauri/
reference/         definições públicas do fabricante para este modelo
```

## Testes

```bash
cargo test
```

```bash
cd app && pnpm typecheck && pnpm lint && pnpm test
```

Com o mouse ou o receptor conectado, `DARMOSHARK_HARDWARE=1 cargo test --test
hardware -- --test-threads=1` também exercita o transporte real; cada escrita é
lida de volta e depois restaurada.

Os testes verificam que os pacotes gerados são idênticos, byte a byte, aos que
o software oficial monta, além das validações de faixa.

## Reproduzir a engenharia reversa

O protocolo saiu do bundle JavaScript do configurador oficial:

```bash
curl -s https://www.darmoshark.cc/ -o index.html
# baixe o main.*.js referenciado e desminifique (jsbeautifier)
```

A definição pública deste modelo está em `reference/`, obtida de:

- `https://launcher.keychron.com/vapi/v2/product/613089042`
- `https://launcher.keychron.com/static/device/613089042/json/v3.json`

O `613089042` é o `vpId`, calculado como `vid << 16 | pid`.

## Aviso

Projeto independente, sem vínculo com Darmoshark, Attack Shark, Motospeed ou
Keychron. Escrever configuração em um dispositivo USB tem risco: use por sua
conta. O comando `reset` restaura o padrão de fábrica caso algo saia do lugar.

## Licença

MIT
