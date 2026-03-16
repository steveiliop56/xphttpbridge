extern crate xplm;

use xplm::plugin::{Plugin, PluginInfo};
use xplm::{debugln, xplane_plugin};

mod config;
mod dataref;
mod server;

struct XPHTTPBridge;

impl Plugin for XPHTTPBridge {
    type Error = std::convert::Infallible;

    fn start() -> Result<Self, Self::Error> {
        debugln!("XPHTTPBridge: Reading config");

        let current_dir_res = std::env::current_dir();
        let current_dir = match current_dir_res {
            Ok(dir) => dir,
            Err(e) => {
                debugln!("XPHTTPBridge: Failed to get current directory: {}", e);
                return Ok(XPHTTPBridge);
            }
        };

        debugln!("XPHTTPBridge: Current directory: {:?}", current_dir);

        let config_path = std::path::Path::new(current_dir.as_path())
            .join("Resources")
            .join("plugins")
            .join("xphttpbridge")
            .join("config.ini");

        let data_ref_path = std::path::Path::new(current_dir.as_path())
            .join("Resources")
            .join("plugins")
            .join("DataRefs.txt");

        debugln!("XPHTTPBridge: Config path: {:?}", config_path);
        debugln!("XPHTTPBridge: Data ref path: {:?}", data_ref_path);

        // we can panic here since the config path is constructed from known-good paths
        let config_res = config::Config::load(config_path.to_str().unwrap());

        let config = match config_res {
            Ok(config) => config,
            Err(e) => {
                debugln!("XPHTTPBridge: Failed to load config: {}", e);
                return Ok(XPHTTPBridge);
            }
        };

        debugln!("XPHTTPBridge: Config loaded: {:?}", config);

        debugln!("XPHTTPBridge: Loading data ref info");

        // same as #43 for unwrapping the data ref path
        let data_refs = dataref::load_and_parse_datarefs(data_ref_path.to_str().unwrap());

        debugln!(
            "XPHTTPBridge: Loaded {:?} data ref entries",
            data_refs.len()
        );

        debugln!("XPHTTPBridge: Starting server");

        std::thread::spawn(|| {
            let runtime_res = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build();

            let runtime = match runtime_res {
                Ok(r) => r,
                Err(e) => {
                    debugln!("XPHTTPBridge: Failed to create runtime: {}", e);
                    return;
                }
            };

            let srv = server::Server::new(config.server, data_refs);

            runtime.block_on(async { srv.start().await })
        });

        Ok(XPHTTPBridge)
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("XPHTTPBridge"),
            signature: String::from("org.steveiliop56.xphttpbridge"),
            description: String::from("A simple HTTP bridge for X-Plane DataRefs."),
        }
    }
}

xplane_plugin!(XPHTTPBridge);
