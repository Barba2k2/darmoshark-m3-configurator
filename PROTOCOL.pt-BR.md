# Darmoshark M3 — protocolo de configuração (contract `dms`)

[English](PROTOCOL.md) · **Português**

Engenharia reversa do bundle do configurador WebHID oficial (`darmoshark.cc`,
app Angular sobre a plataforma Keychron). O mouse vendido como **Attack Shark M3**
se identifica no firmware como **Darmoshark M3**.

## Identidade

| Campo | Valor |
|---|---|
| VID:PID | `0x248A:0xFF12` (Telink) |
| vpId (catálogo) | `613089042` = `vid << 16 \| pid` |
| Definição oficial | `https://launcher.keychron.com/static/device/613089042/json/v3.json` |
| Metadados | `https://launcher.keychron.com/vapi/v2/product/613089042` |
| DPI | 50–26000, níveis padrão 400/800/1600/3200/4800 |
| Report rates | 125 / 500 / 1000 Hz |

## Canais HID

O mouse expõe interfaces diferentes conforme a conexão.

| Interface | Usage page | Função |
|---|---|---|
| 0 / 1 | `0x01`, `0x0C` | mouse / teclado / consumer (genéricas) |
| 2 | `0x8C` | DFU **e** configuração pelo cabo — ver abaixo |
| receptor, iface 1 | `0x8C` | DFU **e** configuração por 2.4GHz — mesmo descriptor |

> **Correção (validada em hardware):** a interface do cabo NÃO é só DFU. Os
> mesmos payloads do `dms` são aceitos como **feature report `0x52`** nela, e
> aplicam de verdade. O engano inicial veio de testar apenas leitura: `setDpi`
> é fire-and-forget no protocolo original, então silêncio é o comportamento
> normal de uma escrita bem-sucedida.

O receptor 2.4GHz (`0x248A:0xFF30`, product string `RF Dongle`) expõe três
interfaces e **nenhuma usage page vendor própria**: a interface de config
carrega exatamente o mesmo descriptor `0x8C` do mouse pelo cabo. As páginas
`0xFF0A` / `0xFFC1` e os output reports `0xB3` / `0xB5` pertencem a outros
modelos do bundle — neste hardware não existem em lugar nenhum, e escrever
neles não chega a nada.

Report ids desse descriptor, idênticos nas duas interfaces:

| Report | Tipo | Tamanho | Função |
|---|---|---|---|
| `0xB1` / `0xB2` | input / output | 32 B | DFU |
| `0x51` | feature | 20 B | identidade pelo cabo, **config pelo receptor** |
| `0x52` | feature | 64 B | config pelo cabo, comandos longos pelo receptor |
| `0x53` | feature | 1000 B | sem uso aqui |
| `0x54` | input | 20 B | confirmações vindas do receptor |

Dois transportes carregam os mesmos payloads:

| Operação | Cabo (feature `0x52`) | Receptor (feature `0x51`) |
|---|---|---|
| Escrita de config | ✅ funciona | ✅ funciona |
| Leitura de config | ❌ devolve sempre o bloco de identidade | ✅ funciona |
| Confirmação | nenhuma, escrita silenciosa | `0xE4` no input `0x54` |
| Níveis de DPI | 5 (só o formato curto) | 5 |
| Leitura do bootloader | a do mouse | a do **próprio receptor** |

Pelo cabo o endpoint USB já opera a 1000 Hz (`ReportInterval = 1000 µs`), então
`setReportRate` só tem efeito observável nos modos sem fio.

## Canal do receptor 2.4GHz (feature `0x51`)

Os comandos são os mesmos payloads de 20 bytes, enviados como feature report
`0x51` (preenchidos com zeros); o que passar disso vai no feature `0x52`, de 64
bytes. As escritas são fire-and-forget, igual ao cabo.

A leitura é o que o cabo não faz. O receptor responde no **input report `0x54`**
com um frame de confirmação, e só então a resposta é buscada no feature report:

```
54 E4 <status> 00
```

O byte depois do status é onde o bundle do fabricante espera o eco do opcode,
mas este receptor sempre manda `00` ali: a confirmação não diz a qual comando
pertence.

| Status | Significado |
|---|---|
| 0 | enfileirado — reenviar o mesmo comando daqui a pouco |
| 1 | pronto — a resposta está esperando no feature report |
| 2 | **sem link com o mouse** — tirar e recolocar o receptor |
| 4 | ocupado — igual a enfileirado |

Uma leitura costuma postar `E4 00` e depois `E4 01` em 10–80 ms. Uma escrita
posta um único `E4 00`, de 300 a 800 ms depois do envio, quando o receptor já
repassou o comando ao mouse — uma leitura feita antes desse frame ainda devolve
o valor antigo. A leitura de bond (opcode 3) não posta confirmação nenhuma; a
resposta simplesmente está no feature report.

Quatro armadilhas, todas caras:

- **Status 2 não se recupera sozinho.** Perdido o link, todo comando responde
  `2` para sempre, inclusive os que funcionariam. O cursor continua se mexendo
  o tempo todo, porque a entrada HID trafega por caminho diferente do canal de
  configuração. Só o replug do receptor traz de volta.
- **O buffer do feature fica velho até a resposta nova chegar.** Ler cedo
  demais devolve a resposta *anterior*, que parece válida mas é de outra
  pergunta. Sempre conferir o eco do opcode — e, nos comandos que endereçam um
  slot (botões, macros), também o eco do índice, já que todos os botões
  compartilham o mesmo opcode.
- **Só o status pronto libera o buffer.** Duas leituras do mesmo opcode ecoam
  os mesmos bytes, então o eco não distingue a resposta nova da anterior. Ler
  o feature report com status `0` devolve o snapshot anterior toda vez; esperar
  o `1`, e reenviar se ele não vier.
- **Confirmações velhas ficam esperando na fila.** Como o frame não nomeia o
  comando, o `E4 00` que uma escrita posta atrasado é indistinguível do que a
  próxima leitura posta. Esvaziar o input `0x54` antes de enviar, e deixar a
  escrita esperar o próprio frame antes de qualquer leitura conferir o valor.

Leituras confirmadas em hardware:

| Opcode | Devolve |
|---|---|
| 3 | vínculo: VID, PID e estado do link do mouse pareado |
| 4 | string de firmware (`2.0.9r`) |
| 5 | nome do produto (`M3 Mouse`) |
| 6 | versão de protocolo, work mode, bateria |
| 7 | **o snapshot de configuração** |

### Snapshot de configuração — opcode `0x07`

Offsets contados a partir do eco do opcode, depois de tirar o report id. É o
`getBaseInfo` do contract `M` — o opcode `0x06` do `dms`, usado pelo cabo, não
responde nada aqui.

| Offset | Campo |
|---|---|
| 0 | eco do opcode (`0x07`) |
| 1 | perfil onboard ativo |
| 2 / 3 / 4 | slot usb / 2.4GHz / bluetooth — nibble baixo é o índice de DPI ativo, nibble alto o índice de report rate |
| 5..14 | cinco uint16 little-endian de DPI |
| 15 | bits do sensor, 1 = ligado — `[1:0]` lift-off, `[2]` wave, `[3]` line, `[4]` motion, `[6]` scroll, `[7]` eSports; `[5]` vem ligado no M3 e não corresponde a nada conhecido |
| 16 | quantidade de níveis de DPI habilitados |
| 17 | debounce do clique, em milissegundos |
| 18 | timer de sono, em minutos (só leitura aqui, ver opcode 10) |

Exemplo lido de um M3 com firmware `2.0.9r`:

```
51 07 00 13 13 03 90 01 20 03 40 06 80 0c c0 12 35 05 08 00 00
```

perfil 0 · nível ativo 3 · índice de rate 1 · 400/800/1600/3200/4800 ·
debounce 8 ms · lift-off 1.

> O nibble de rate é o índice da frequência na lista do perfil
> (125/500/1000 Hz). Medido pelo receptor cronometrando os próprios relatórios
> de entrada do mouse em movimento contínuo: índice 0 chega a cada 8,00 ms, 1 a
> cada 2,00 ms, 2 a cada 1,00 ms. O receptor também aceita o índice 3, que o
> perfil do M3 não declara; do 4 em diante é ignorado.

## Comandos

Payload sem checksum; o report id define o tamanho.

### Ler configuração — `0xB3`, payload 63 B

```
[0] = 0x06
```

Resposta (input report, `[0]` ∈ {`0x05`, `0x06`}):

| Offset | Conteúdo |
|---|---|
| 1 | profile |
| 2 / 3 / 4 | slot USB / 2.4G / BT — nibble alto = report rate, baixo = índice de DPI |
| 5..14 | 5 níveis de DPI, uint16 little-endian |
| 15 | flags de sistema (lod, wave, line, motion, scroll, eSports) |
| 16 | número de níveis ativos (`gears`) |
| 17 | delay |
| 18 | sleep, em minutos |
| 19 | bateria — bit 7 = carregando, bits 0–6 = porcentagem |
| 20..25 | níveis 6–8 de DPI |
| 27/28/29 | scroll speed / inertia / spl |
| 30..39 | debounce |

### Escrever DPI (≤5 níveis) — `0xB5`, payload 20 B

```
[0]      = 0x40
[1..3]   = índice do nível ativo (repetido 3x)
[4..13]  = 5 × uint16 little-endian
[14]     = quantidade de níveis habilitados
```

Exemplo — 400/800/1600/3200/4800, ativo no índice 2:

```
40 02 02 02 90 01 20 03 40 06 80 0c c0 12 05 00 00 00 00 00
```

### Escrever DPI (>5 níveis) — `0xB3`, payload 63 B

```
[0]      = 0x44
[1..3]   = índice do nível ativo
[4]      = quantidade de níveis
[5..]    = N × uint16 little-endian
```

## Variante `dms_v2`

O configurador detecta o contract lendo um report inicial: se `reply[6] == 21`
e `reply[7] == 25`, usa `dms_v2`, senão `dms`. O M3 usa `dms` — o `dms_v2` tem
opcodes próprios e não está implementado aqui.

## Canal DFU (cabo, usage page `0x8C`)

Enquadramento: output report `0xB2`, resposta em input report `0xB1`,
podendo vir fragmentada em vários pacotes de 32 B.

```
[0] 0xAA          header
[1] 0x55 / 0x56   send no-ack / send ack
[2] len
[3] ~len          complemento de 1 de len
[4] kind
[5] comando
[6] checksum (soma dos bytes de parâmetro)
```

Comandos de leitura confirmados no M3:

| Cmd | kind | Retorna |
|---|---|---|
| 96 | 1 | módulo, firmware e hardware |
| 97 | 2 | revisão do bootloader DFU |

Resposta do cmd 96, após remontar os fragmentos (payload a partir do offset 5
do primeiro pacote):

| Offset | Campo |
|---|---|
| 4..13 | modelo do módulo (`MOTO_M3`) |
| 16..25 | versão de firmware (`2.0.9r`) |
| 26..35 | versão de hardware (`1.0.0`) |

## Canal de identificação (cabo, feature report `0x51`)

Round trip por feature report: escreve o comando, lê a resposta no mesmo id.

Comando `0x06` — resposta (após o report id ecoado):

| Offset | Campo |
|---|---|
| 0 | echo do opcode |
| 1 | status (1 = ok) |
| 3..4 | VID little-endian |
| 5..6 | PID little-endian |
| 7..8 | firmware (`0x0209` → 2.0.9) |
| 10 | bateria em % |

O opcode `0x03` (identify do receptor) não é respondido pelo mouse — é
comando do dongle. Os report ids `0xB3`/`0xB5` não existem como feature
report neste canal: retornam apenas zeros.

## Tabela de comandos do `dms`

Opcode no byte 0 do payload. Roteamento: `0xB5` = payload de 20 B,
`0xB3` = payload de 63 B. O ACK volta como `0xE4 <status> <opcode>`,
com `status = 0` em caso de sucesso.

| Opcode | Nome | Report | Função |
|---|---|---|---|
| 2 | `getProtocol` | 0xB5 | versão do protocolo |
| 3 | `getBondInfo` | 0xB5 | estado do pareamento com o receptor |
| 4 | `getDeviceString` | 0xB3 | string do dispositivo |
| 5 | `getMouseInfo` | 0xB5 | info básica |
| 6 | `getMouseExtInfo` | 0xB3 | **snapshot completo da config** |
| 10 | `deviceTime` | 0xB5 | timer de sleep — `[1]` 1=set 2=get, `[2]` minutos. Só pelo cabo: o contrato do receptor nunca o envia, uma escrita pelo receptor deixa o byte 18 do snapshot igual, e um `get` nunca fica pronto |
| 11 | `pairButton` | 0xB5 | pareamento |
| 14 | `profileSwitch` | 0xB5 | troca de perfil — `[1]` índice |
| 15 | `driverConfigRecovery` | 0xB5 | reset; `[1]=63` = padrão de fábrica |
| 35 / 36 | `get/setLightEffectParam` | 0xB5 | iluminação |
| 64 | `setDpi` | 0xB5 | DPI, até 5 níveis |
| 65 | `setReportRate` | 0xB5 | `[1..2]` índice da taxa, repetido; o resto zero. Os dois contratos do fabricante chamam o índice de `level` — não é nível de DPI. Confirmado pelo receptor |
| 66 | `setSensorLiftCutoff` | 0xB5 | `[1]` LOD, `[2]` wave, `[3]` line, `[4]` motion, `[6]` scroll, `[7]` eSports. wave/line/motion: 1 = ligado, 2 = desligado; scroll/eSports: 2 = ligado, 1 = desligado (confirmado um por vez contra o byte 15 do snapshot). Toda escrita define os cinco, então mudar o lift-off exige reenviar os valores guardados |
| 67 | `setButtonDebounce` | 0xB5 | `[1]` ms |
| 68 | `setDpiExtended` | 0xB3 | DPI, mais de 5 níveis |
| 69 | `setScroll` | 0xB5 | `[1]` speed, `[2]` inertia, `[3]` spl |
| 82 | `setButtonConfig` | 0xB3 | `[1]` botão, `[3]` tipo, `[4..]` dados |
| 83 / 84 | `setMacroName` / `setMacroData` | 0xB3 | macros |
| 97 / 98 | `getAllButtonConfig` / `getButtonConfig` | 0xB3 | leitura de botões |
| 99 / 100 | `getMacroName` / `getMacroData` | 0xB3 | leitura de macros |
| 113 / 114 | `longDataTransfer` / `longDataFlowControl` | 0xB3 | transferência longa |

### Tipos de atribuição de botão

`0` remove · `1` mouse · `2` teclado · `3` mídia · `4` macro · `5` dpi ·
`6` light · `7` gameReinforce · `8` shortCut · `9` disable · `10` profileSwitch

### Eventos assíncronos (input reports)

| `[0]` | Evento |
|---|---|
| 225 | iluminação alterada |
| 226 | base alterada — `[1]` workMode, `[2]` conexão, `[3..4]` bateria, `[5..7]` dpi/rate/nível |
| 229 | perfil alterado — `[1]` novo perfil |

### Opcodes de leitura do canal `0x51`

Estrutura da resposta: `[0]` opcode ecoado, `[1]` tamanho, `[2..]` dados.

| Opcode | Retorno | Exemplo |
|---|---|---|
| `0x02` | sem resposta | — |
| `0x03` | sem resposta (comando do dongle) | — |
| `0x04` | string de firmware | `2.0.9r` |
| `0x05` | nome do produto | `M3 Mouse` |
| `0x06` | identidade + bateria | VID, PID, fw, `%` |

Nenhum deles carrega estado de DPI. Pelo cabo o mouse é somente leitura de
identificação — configurar exige o canal `0xFFC1` do dongle.


## Transporte pelo cabo (feature report `0x52`)

Os payloads do `dms` — idênticos, byte a byte — são enviados como feature
report `0x52`, preenchidos com zeros até 64 bytes.

```
send_feature_report(0x52, payload + padding_ate_64)
```

Confirmado em hardware no Darmoshark M3 (fw 2.0.9r), validando pela cor do LED
indicador a cada mudança de nível:

| Comando | Enviado | LED resultante |
|---|---|---|
| `setDpi` nível 1 | `40 01 01 01 ...` | azul (800 DPI) |
| `setDpi` nível 4 | `40 04 04 04 ...` | amarelo (4800 DPI) |
| `setDpi` nível 0 | `40 00 00 00 ...` | vermelho (400 DPI) |

Canais testados que **não** funcionam para config: feature `0x51`, output `0xB2`
com payload cru, e output `0xB2` com o payload encapsulado em `AA/55` (kinds
1, 2 e 3). Só o `0x52` encaminha ao core.

A leitura pelo `0x52` tem handler fixo: qualquer opcode devolve o bloco de
identidade (VID, PID, firmware, bateria), nunca o snapshot de configuração.

### Limite de 5 níveis pelo cabo

Só o **formato curto** (`setDpi`, opcode `0x40`) atravessa o canal do cabo. O
formato estendido (`setDpiExtended`, opcode `0x44`, usado para 6 ou mais
níveis) é aceito sem erro mas **ignorado pelo firmware** — testado com 6 e com
5 níveis, nenhum dos dois surtiu efeito.

Como o pacote curto reserva os bytes `[4..13]` para cinco `uint16`, o teto pelo
cabo é de **5 níveis de DPI**. Cada um aceita qualquer valor dentro de
50–26000; o que não dá é ter um sexto.
