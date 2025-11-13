//!/////// START OF FILE //////////
//! GOBSG
//!
//! Copyright (c) 2025 Gistyr LLC
//!
//! Licensed under the PolyForm Small Business License 1.0.0
//! See LICENSES/LICENSE-POLYFORM-SMALL-BUSINESS.md or https://polyformproject.org/licenses/small-business/1.0.0
//!
//! Required Notice: Copyright Gistyr LLC (https://gistyr.dev)
//!
//! For full licenses see:
//! LICENSES/
//!
//! ---------------------------------------- //

use better_logger::{LoggerSettings, NetworkFormat, logger::debugx};
use std::process::exit;
use std::fs::read_to_string;
use serde::Deserialize;

pub const LOGGING_CONFIG_PATH: &str = "logging-config.toml";
pub const MAIN_CONFIG_PATH: &str = "main-config.toml";

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ConfigNetworkFormat {
    PlainText,
    JsonText { field: String },
}

impl From<ConfigNetworkFormat> for NetworkFormat {
    fn from(v: ConfigNetworkFormat) -> Self {
        match v {
            ConfigNetworkFormat::PlainText => NetworkFormat::PlainText,
            ConfigNetworkFormat::JsonText { field } => NetworkFormat::JsonText { field },
        }
    }
}

#[derive(Deserialize)]
pub struct LoggingConfiguration {
    pub terminal_logs: bool,
    pub terminal_log_lvl: String,
    pub wasm_logging: bool,
    pub file_logs: bool,
    pub file_log_lvl: String,
    pub log_file_path: String,
    pub network_logs: bool,
    pub network_log_lvl: String,
    pub network_endpoint_url: String,
    pub network_format: ConfigNetworkFormat,
    pub debug_extra: bool,
    pub async_logging: bool,
}

pub fn new_logger_settings() -> LoggerSettings {
    let config: LoggingConfiguration = {
        match read_to_string(LOGGING_CONFIG_PATH) {
            Ok(raw) => {
                match toml::from_str(&raw) {
                    Ok(config) => config,
                    Err(error) => {
                        eprintln!("toml::from_str failed: {:?}", error);
                        exit(1);    
                    }
                }
            }
            Err(error) => {
                eprintln!("read_to_string({:?}) failed: {:?}", LOGGING_CONFIG_PATH, error);
                exit(1);                
            }
        }
    };

    return LoggerSettings {
        terminal_logs: config.terminal_logs,
        terminal_log_lvl: config.terminal_log_lvl,
        wasm_logging: config.wasm_logging,
        file_logs: config.file_logs,
        file_log_lvl: config.file_log_lvl,
        log_file_path: config.log_file_path,
        network_logs: config.network_logs,
        network_log_lvl: config.network_log_lvl,
        network_endpoint_url: config.network_endpoint_url,
        network_format: config.network_format.into(),
        debug_extra: config.debug_extra,
        async_logging: config.async_logging,
    };
}

#[derive(Deserialize)]
pub struct ReadConfiguration {
    pub this_server_url: String,
    pub cookie_name: String,
    pub cookie_domain: String,
    pub secret_cookie_hex_key: String,
    pub requesting_client_url: String,
    pub issuer_url: String,
    pub logout_url: String,
    pub client: String,
    pub client_secret: String,

    pub listen_address: Option<String>,
    pub listen_port: Option<u16>,
    pub workers: Option<usize>,
    pub redis_address: Option<String>,
    pub heartbeat_logging: Option<bool>,
    pub heartbeat_interval_hours: Option<u16>,
    pub machine_name: Option<String>,
    pub container_name: Option<String>,
    pub provider: Option<String>,
    pub keep_alive_time_secs: Option<u64>,
    pub client_request_timeout_secs: Option<u64>,
    pub client_disconnect_timeout_secs: Option<u64>,
    pub max_connections: Option<usize>,
    pub early_refresh_skew_secs: Option<i64>,
    pub user_details_fail_when_not_authenticated: Option<bool>,
    pub default_username: Option<String>,
    pub default_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MainConfiguration {
    pub this_server_url: String,
    pub cookie_name: String,
    pub cookie_domain: String,
    pub secret_cookie_hex_key: String,
    pub requesting_client_url: String,
    pub issuer_url: String,
    pub logout_url: String,
    pub client: String,
    pub client_secret: String,

    pub listen_address: String,
    pub listen_port: u16,
    pub workers: usize,
    pub redis_address: String,
    pub heartbeat_logging: bool,
    pub heartbeat_interval_hours: u16,
    pub machine_name: String, 
    pub container_name: String,
    pub provider: String,
    pub keep_alive_time_secs: u64,
    pub client_request_timeout_secs: u64,
    pub client_disconnect_timeout_secs: u64,
    pub max_connections: usize,
    pub early_refresh_skew_secs: i64,
    pub user_details_fail_when_not_authenticated: bool,
    pub default_username: String,
    pub default_user_id: String,
}

impl MainConfiguration {
    pub fn new() -> MainConfiguration {
        let config: ReadConfiguration = {
            match read_to_string(MAIN_CONFIG_PATH) {
                Ok(raw) => {
                    match toml::from_str(&raw) {
                        Ok(config) => config,
                        Err(error) => {
                            eprintln!("toml::from_str failed: {:?}", error);
                            exit(1);    
                        }
                    }
                }
                Err(error) => {
                    eprintln!("read_to_string({:?}) failed: {:?}", MAIN_CONFIG_PATH, error);
                    exit(1);                
                }
            }
        };

        let listen_address = match config.listen_address {
            Some(address) => address,
            None => "0.0.0.0".to_string(),
        };
        let listen_port = match config.listen_port {
            Some(port) => port,
            None => 3090,
        };
        let workers = match config.workers {
            Some(workers) => workers,
            None => num_cpus::get().max(1),
        };
        let redis_address = match config.redis_address {
            Some(redis) => redis,
            None => "redis://127.0.0.1:6379".to_string(),
        };
        let heartbeat_logging = match config.heartbeat_logging {
            Some(heart) => heart,
            None => false,
        };
        let heartbeat_interval_hours = match config.heartbeat_interval_hours {
            Some(interval) => interval,
            None => 12,
        };
        let machine_name = match config.machine_name {
            Some(name) => name,
            None => "machine".to_string(),
        };
        let container_name = match config.container_name {
            Some(name) => name,
            None => "container".to_string(),
        };
        let provider = match config.provider {
            Some(provider) => provider,
            None => "provider".to_string(),
        };
        let keep_alive_time_secs = match config.keep_alive_time_secs {
            Some(alive) => alive,
            None => 75,
        };
        let client_request_timeout_secs = match config.client_request_timeout_secs {
            Some(timeout) => timeout,
            None => 30,
        };
        let client_disconnect_timeout_secs = match config.client_disconnect_timeout_secs {
            Some(timeout) => timeout,
            None => 5,
        };
        let max_connections = match config.max_connections {
            Some(connections) => connections,
            None => 25000,
        };
        let early_refresh_skew_secs =  match config.early_refresh_skew_secs {
            Some(skew) => skew,
            None => 120,
        };
        let user_details_fail_when_not_authenticated = match config.user_details_fail_when_not_authenticated {
            Some(fail_not_auth) => fail_not_auth,
            None => true,
        };
        let default_username = match config.default_username {
            Some(username) => username,
            None => "0".to_string(),
        };
        let default_user_id = match config.default_user_id{
            Some(id) => id,
            None => "0".to_string(),
        };

        let main_config_testing = MainConfiguration {
            this_server_url: config.this_server_url,
            cookie_name: config.cookie_name,
            cookie_domain: config.cookie_domain,
            secret_cookie_hex_key: config.secret_cookie_hex_key,
            requesting_client_url: config.requesting_client_url,
            issuer_url: config.issuer_url,
            logout_url: config.logout_url,
            client: config.client,
            client_secret: config.client_secret,

            listen_address: listen_address,
            listen_port: listen_port,
            workers: workers,
            redis_address: redis_address,
            heartbeat_logging: heartbeat_logging,
            heartbeat_interval_hours: heartbeat_interval_hours,
            machine_name: machine_name,
            container_name: container_name,
            provider: provider,
            keep_alive_time_secs: keep_alive_time_secs,
            client_request_timeout_secs: client_request_timeout_secs,
            client_disconnect_timeout_secs: client_disconnect_timeout_secs,
            max_connections: max_connections,
            early_refresh_skew_secs: early_refresh_skew_secs,
            user_details_fail_when_not_authenticated: user_details_fail_when_not_authenticated,
            default_username: default_username,
            default_user_id: default_user_id,
        };
        debugx!("{:?}", main_config_testing);
        return main_config_testing;
    }
}

////////// END OF FILE //////////