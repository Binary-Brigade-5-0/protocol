use std::{net::SocketAddr, sync::OnceLock, time::SystemTime};

static SETTINGS: OnceLock<Settings> = OnceLock::new();

#[derive(Debug)]
pub struct Settings {
    addr: SocketAddr,
    dmap_capacity: usize,
    start_time: SystemTime,
}

#[allow(unused, dead_code)]
impl Settings {
    const HELP_STRING: &'static str = r#"
usage: etron [arguments]

arguments:
    -a --address  <socket address> set the socket address for the server
                  default: 127.0.0.1:3000
"#;

    /// Gets a static instance of the settings variable from std::env::args,
    /// PANICKS: exits with exit_code 1 if the arguments cannot be parsed
    pub fn instance() -> &'static Self {
        SETTINGS.get_or_init(Self::from_env)
    }

    pub fn try_from_env() -> Result<Self, pico_args::Error> {
        let mut pargs = pico_args::Arguments::from_env();
        let addr = pargs
            .opt_value_from_str(["-a", "--addr"])?
            .unwrap_or(SocketAddr::from(([0u8; 4], 3000)));

        let dmap_capacity = pargs.opt_value_from_str("--cap")?.unwrap_or(1024);

        for argument in pargs.finish() {
            eprintln!("warning: unknown argument: {argument:?}");
        }

        Ok(Self {
            addr,
            dmap_capacity,
            start_time: SystemTime::now(),
        })
    }

    pub fn from_env() -> Self {
        match Self::try_from_env() {
            Ok(settings) => settings,
            Err(error) => {
                eprint!("error: {error}{}", Self::HELP_STRING);
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
}
