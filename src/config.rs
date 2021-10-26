use getopts::Options;

pub struct Config {
    pub token: String,
    pub proxy: Option<String>,
    pub webhook: u16,
    pub trigger_with_command: bool,
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut opts = Options::new();

        opts.optopt(
            "t",
            "token",
            "(required) set Telegram Bot HTTP API token",
            "TOKEN",
        );
        opts.optopt(
            "p",
            "proxy",
            "set proxy (supported: http, https, socks5)",
            "PROXY",
        );
        opts.optopt(
            "w",
            "webhook-port",
            "set webhook port (1 ~ 65535) and run bot in webhook mode",
            "WEBHOOK_PORT",
        );
        opts.optflag(
            "c",
            "trigger-with-command",
            "trigger translate with command rather than any text message",
        );
        opts.optflag("h", "help", "print this help menu");

        let usage = opts.usage(&format!("Usage: {} -t TELEGRAM-TOKEN [options]", args[0]));

        let matches = opts.parse(&args[1..]).map_err(|e| e.to_string())?;

        if matches.opt_present("h") {
            return Err(usage);
        }

        Ok(Self {
            token: matches
                .opt_str("t")
                .ok_or_else(|| String::from("Telegram Bot HTTP API token not set"))?,
            proxy: matches.opt_str("p"),
            webhook: matches
                .opt_str("w")
                .map_or_else(|| 0, |port| port.parse().unwrap_or(0)),
            trigger_with_command: matches.opt_present("c"),
        })
    }
}
