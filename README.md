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
/toggle_command - Toggle translate-all-messages mode for the current chat (default: off)
/toggle_mention - Toggle if I should only react to non-command messages that mentions me in the group. You still need to @ me when using command (default: on)
/about - About this bot
/help - Get this help message
```

## Build

```bash
$ git clone https://github.com/EAimTY/bing-dict-telegram-bot && cd bing-dict-telegram-bot
$ cargo build --release
```

## License

GNU General Public License v3.0
