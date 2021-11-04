# bing-dict-telegram-bot

A Telegram bot uses Bing Dictionary to translate words and phrases from Chinese to English or English to Chinese.

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

In chat:

```
/dict [word / phrase] - Translate a word or phrase
/toggle - Switch to the mode of translating all messages in the current chat (@ the bot is required if it is a group)
/about - About this bot
/help - Get this help message
```

## Build

Rust Nightly is required.

```bash
$ git clone https://github.com/EAimTY/bing-dict-telegram-bot && cd bing-dict-telegram-bot
$ cargo build --release
```

## License

GNU General Public License v3.0
