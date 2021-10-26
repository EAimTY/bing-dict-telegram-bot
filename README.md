# bing-dict

A Telegram bot translates words from Chinese to English or from English to Chinese using Bing Dict.

## Usage

```
Usage: bing-dict-telegram-bot -t TELEGRAM-TOKEN [options]

Options:
    -t, --token TOKEN   (required) set Telegram Bot HTTP API token
    -p, --proxy PROXY   set proxy (supported: http, https, socks5)
    -w, --webhook-port WEBHOOK_PORT
                        set webhook port (1 ~ 65535) and run bot in webhook
                        mode
    -c, --trigger-with-command 
                        trigger translate with command rather than any text
                        message
    -h, --help          print this help menu
```

## Build

Rust nighlty is required.

```bash
$ git clone https://github.com/EAimTY/bing-dict-telegram-bot && cd bing-dict-telegram-bot
$ cargo build --release
```

## License

GNU General Public License v3.0
