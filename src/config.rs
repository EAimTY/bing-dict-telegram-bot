use anyhow::{anyhow, bail, Result};
use getopts::Options;
use jsave::RwLock;
use nohash_hasher::BuildNoHashHasher;
use reqwest::Proxy;
use std::{collections::HashSet, fs::OpenOptions};

pub struct Config {
    pub token: String,
    pub database: RwLock<HashSet<i64, BuildNoHashHasher<i64>>>,
    pub webhook_port: Option<u16>,
    pub proxy: Option<Proxy>,
}

pub struct ConfigBuilder<'cfg> {
    opts: Options,
    program: Option<&'cfg str>,
}

impl<'cfg> ConfigBuilder<'cfg> {
    pub fn new() -> Self {
        let mut opts = Options::new();

        opts.reqopt(
            "t",
            "token",
            "Set the Telegram Bot HTTP API token (required)",
            "TOKEN",
        );

        opts.reqopt(
            "d",
            "database",
            "Set the database JSON file location (required)",
            "DATABASE_FILE",
        );

        opts.optopt(
            "w",
            "webhook-port",
            "Run in webhook mode listening port (1 ~ 65535)",
            "WEBHOOK_PORT",
        );

        opts.optopt(
            "",
            "proxy",
            "Set proxy  (supported: http, https, socks5)",
            "PROXY",
        );

        opts.optflag("v", "version", "Print the version");
        opts.optflag("h", "help", "Print this help menu");

        Self {
            opts,
            program: None,
        }
    }

    pub fn get_usage(&self) -> String {
        self.opts.usage(&format!(
            "Usage: {} [options]",
            self.program.unwrap_or(env!("CARGO_PKG_NAME"))
        ))
    }

    pub fn parse(&mut self, args: &'cfg [String]) -> Result<Config> {
        self.program = Some(&args[0]);

        let matches = self
            .opts
            .parse(&args[1..])
            .map_err(|err| anyhow!("{err}\n\n{}", self.get_usage()))?;

        if !matches.free.is_empty() {
            bail!(
                "Unexpected argument: {}\n\n{}",
                matches.free.join(", "),
                self.get_usage()
            );
        }

        if matches.opt_present("v") {
            bail!("{}", env!("CARGO_PKG_VERSION"));
        }

        if matches.opt_present("h") {
            bail!("{}", self.get_usage());
        }

        let token = unsafe { matches.opt_str("t").unwrap_unchecked() };

        let database = {
            let path = unsafe { matches.opt_str("d").unwrap_unchecked() };

            if OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
                .is_ok()
            {
                RwLock::init_with(HashSet::with_hasher(BuildNoHashHasher::default()), path)?
            } else {
                RwLock::init(path)?
            }
        };

        let webhook_port = if let Some(port) = matches.opt_str("w") {
            let port = port
                .parse()
                .map_err(|err| anyhow!("{err}\n\n{}", self.get_usage()))?;

            if port == 0 {
                bail!(
                    "Port 0 cannot be used as the webhook port\n\n{}",
                    self.get_usage()
                );
            }

            Some(port)
        } else {
            None
        };

        let proxy = if let Some(proxy) = matches.opt_str("proxy") {
            Some(Proxy::all(&proxy).map_err(|err| anyhow!("{err}\n\n{}", self.get_usage()))?)
        } else {
            None
        };

        Ok(Config {
            token,
            database,
            webhook_port,
            proxy,
        })
    }
}
