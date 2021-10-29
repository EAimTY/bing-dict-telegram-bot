# bing-dict

A Telegram bot uses Bing Dictionary to translate words from Chinese to English or English to Chinese.

## Usage

```
Usage: bing-dict-telegram-bot -t TELEGRAM-TOKEN [options]

Options:
    -t, --token TOKEN   (required) set Telegram Bot HTTP API token
    -p, --proxy PROXY   set proxy (supported: http, https, socks5)
    -w, --webhook-port WEBHOOK_PORT
                        set webhook port (1 ~ 65535) and run bot in webhook
                        mode
    -h, --help          print this help menu
```

## Build

```bash
$ git clone https://github.com/EAimTY/bing-dict-telegram-bot && cd bing-dict-telegram-bot
$ cargo build --release
```

## License

GNU General Public License v3.0
