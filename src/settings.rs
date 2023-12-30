use std::{net::SocketAddr, sync::OnceLock, time::SystemTime};

static SETTINGS: OnceLock<Settings> = OnceLock::new();

#[derive(Debug)]
pub struct Settings {
    addr: SocketAddr,
    dmap_capacity: usize,
    start_time: SystemTime,
    pg_uri: String,
}

const HELP_STRING: &str = r#"
usage: etron [arguments]

arguments:
    -a --address  <socket address>  set the socket address for the server
                  default: 127.0.0.1:3000

       --cap      <buffer capacity> set the mailbox buffer initial capacity
                  default: 1024(release) 0(debug)

    -p --postgres <postgres url>   set the postgres connection string
                  default: env:DATABASE_URL
"#;

#[allow(unused, dead_code)]
impl Settings {
    /// Gets a static instance of the settings variable from std::env::args,
    /// PANICS: exits with exit_code 1 if the arguments cannot be parsed
    pub fn instance() -> &'static Self {
        SETTINGS.get_or_init(Self::from_env)
    }

    pub fn try_from_env() -> anyhow::Result<Self> {
        let mut pargs = pico_args::Arguments::from_env();

        let addr = pargs
            .opt_value_from_str(["-a", "--addr"])?
            .unwrap_or(SocketAddr::from(([0u8; 4], 3000)));

        let Some(pg_uri) = pargs
            .opt_value_from_str(["-p", "--postgres"])?
            .or(std::env::var("DATABASE_URL").ok())
        else {
            anyhow::bail!("missing postgres url...\n{HELP_STRING}");
        };

        let dmap_capacity = pargs.opt_value_from_str("--cap")?.unwrap_or(1024);

        if let Some(next) = pargs.finish().first() {
            anyhow::bail!("unknown argument: {}", next.to_str().unwrap());
        }

        Ok(Self {
            pg_uri,
            addr,
            dmap_capacity,
            start_time: SystemTime::now(),
        })
    }

    pub fn from_env() -> Self {
        match Self::try_from_env() {
            Ok(settings) => settings,
            Err(error) => {
                eprint!("error: {error}{HELP_STRING}");
                std::process::exit(1);
            },
        }
    }
}

// Getter functions
impl Settings {
    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn dmap_capacity(&self) -> usize {
        self.dmap_capacity
    }

    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }

    pub fn pg_uri(&self) -> &str {
        self.pg_uri.as_str()
    }
}
