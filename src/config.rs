use ini::Ini;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub address: String,
}

#[derive(Debug)]
pub struct Config {
    pub server: ServerConfig,
}

impl Config {
    pub fn load(config_path: &str) -> Result<Self, ini::Error> {
        let mut config = Config {
            server: ServerConfig {
                port: 48000,
                address: String::from("127.0.0.1"),
            },
        };

        let read_res = Ini::load_from_file(config_path);

        let conf_ini = match read_res {
            Ok(ini) => ini,
            Err(e) => return Err(e),
        };

        let server_res = conf_ini.section(Some("server"));

        let server = match server_res {
            Some(section) => ServerConfig {
                port: section
                    .get("port")
                    .and_then(|v| v.parse::<u16>().ok())
                    .unwrap_or(48000),
                address: section.get("address").unwrap_or("127.0.0.1").to_string(),
            },
            None => config.server.clone(),
        };

        config.server = server;

        return Ok(config);
    }
}
