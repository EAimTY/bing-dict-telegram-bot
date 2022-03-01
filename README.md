# bing-dict-telegram-bot

A Telegram bot using Bing Dictionary to translate words and phrases between English and Chinese

## Usage

```
Usage: bing-dict-telegram-bot [options]

Options:
    -t, --token TOKEN   Set the Telegram Bot HTTP API token (required)
    -d, --database DATABASE_FILE
                        Set the database JSON file location (required)
    -w, --webhook-port WEBHOOK_PORT
                        Run in webhook mode listening port (1 ~ 65535)
        --proxy PROXY   Set proxy (supported: http, https, socks5)
    -v, --version       Print the version
    -h, --help          Print this help menu
```

In chat:

```
/dict [word / phrase] - Translate a word or phrase
/toggle - Switch to the mode of translating all messages in the current chat (@bot_username is required if it is a group)
/about - About this bot
/help - Get this help message
```

Or using inline mode (inline mode for bot needs to be enabled in `@botfather` settings):

Just enter `@bot_username word / phrase` in any chat and select the result

## Build

Require Rust 1.59 or above

```bash
$ git clone https://github.com/EAimTY/bing-dict-telegram-bot && cd bing-dict-telegram-bot
$ cargo build --release
```

## License

GNU General Public License v3.0
