# pix-code

A Telegram bot for generating Pix payment QR codes (Brazilian instant payment system).

## Overview

This project implements a Telegram bot that generates **Pix QR codes** using the **EMV® QR Code Specification for Payment Systems**. Users interact with the bot through a conversational flow to provide their Pix key, transaction amount, merchant name, and city — the bot then returns a QR code image that can be scanned by any Brazilian banking app.

## Quick Start

### Prerequisites

- Rust 1.78+ (edition 2024)
- Telegram Bot Token (from [@BotFather](https://t.me/BotFather))

### Create a Bot with BotFather

1. Open Telegram and message [@BotFather](https://t.me/BotFather)
2. Send `/newbot`
3. Choose a display name (e.g., "My Pix Bot")
4. Choose a username ending in `bot` (e.g., `mypixbot`)
4. BotFather replies with a token — **copy it** (format: `123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ`)

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TELOXIDE_TOKEN` | Yes | Bot token from BotFather |
| `DATABASE_FILE` | No | SQLite database file path (default: `bot_state.sqlite`) |

```bash
export TELOXIDE_TOKEN="123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ"
# optional
export DATABASE_FILE="/data/db.sqlite"
```

### Run locally

```bash
cargo run --release
```

### Docker

```bash
docker build -t pix-code .
docker run -e TELOXIDE_TOKEN pix-code
```

## Bot Commands

| Command | Description |
|---------|-------------|
| `/start` | Initial setup (name → city → Pix key) |
| `/code <amount>` | Generate Pix QR code (e.g., `/code 25.50`) |
| `/setname <name>` | Update merchant name |
| `/setcity <city>` | Update merchant city |
| `/setkey <key>` | Update Pix key |
| `/help` | Show command list |

## Dialogue Flow

### First-time Setup
```
User: /start
Bot:  "Para começar digite o seu nome? Ele sera visivel ao pagador"
User: "João da Silva"
Bot:  "Me envie a cidade que está cadastrada sua conta bancaria"
User: "São Paulo"
Bot:  "Qual a sua chave pix?"
User: "joao@email.com"
Bot:  [shows command list]
```

### After Setup (Configured State)
User sends commands directly:
| Command | Example | Action |
|---------|---------|--------|
| `/code <amount>` | `/code 25.50` | Generate Pix QR code for amount |
| `/setname <name>` | `/setname Maria Silva` | Update merchant name |
| `/setcity <city>` | `/setcity Rio de Janeiro` | Update merchant city |
| `/setkey <key>` | `/setkey maria@email.com` | Update Pix key |
| `/help` | `/help` | Show command list |

The bot stores `full_name`, `city`, and `key` set in the initial conversation. 

## EMV Payload Structure

The generated payload follows the Brazilian Pix EMV specification:

| Tag | Field | Example |
|-----|-------|---------|
| 00 | Payload Format Indicator | `01` |
| 26 | Merchant Account Information (GUI `br.gov.bcb.pix`) | Nested: 00=GUI, 01=Key, 02=Description |
| 52 | Merchant Category Code | `0000` |
| 53 | Transaction Currency | `986` (BRL) |
| 54 | Transaction Amount | `25.00` |
| 58 | Country Code | `BR` |
| 59 | Merchant Name | `JOÃO DA SILVA` |
| 60 | Merchant City | `SAO PAULO` |
| 62 | Additional Data Fields | `***` (transaction id) |
| 63 | CRC16 Checksum | Computed over all above |

## License

AGPL-3.0 — see [LICENSE](LICENSE) for details.

