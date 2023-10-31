use std::{net::SocketAddr, sync::OnceLock};

static SETTINGS: OnceLock<Settings> = OnceLock::new();

#[derive(Debug)]
pub struct Settings {
    addr: SocketAddr,
}

#[allow(unused, dead_code)]
impl Settings {
    const HELP_STRING: &str = r#"
usage: etron [arguments]

arguments:
    -a --addr    <socket address> set the socket address for the server
                 default: 127.0.0.1:3000
"#;

    pub fn instance() -> &'static Self {
        SETTINGS.get_or_init(Self::from_env)
    }

    pub fn try_from_env() -> Result<Self, pico_args::Error> {
        let mut pargs = pico_args::Arguments::from_env();
        let addr = pargs
            .opt_value_from_str(["-a", "--addr"])?
            .unwrap_or(SocketAddr::from(([0u8; 4], 3000)));

        Ok(Self { addr })
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
}
