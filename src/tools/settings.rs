//!/////// START OF FILE //////////
//! GOBSG
//!
//! Copyright (c) 2026 Gistyr LLC
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

use better_logger::{LoggerSettings, NetworkFormat};
use std::fs::read_to_string;
use serde::Deserialize;

pub(crate) const LOGGING_CONFIG_PATH: &str = "logging-config.toml";
pub(crate) const MAIN_CONFIG_PATH: &str = "main-config.toml";

#[derive(Deserialize)]
#[serde(tag = "type")]
pub(crate) enum ConfigNetworkFormat {
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
pub(crate) struct LoggingConfiguration {
    pub(crate) terminal_logs: bool,
    pub(crate) terminal_log_lvl: String,
    pub(crate) file_logs: bool,
    pub(crate) file_log_lvl: String,
    pub(crate) log_file_path: String,
    pub(crate) network_logs: bool,
    pub(crate) network_log_lvl: String,
    pub(crate) network_endpoint_url: String,
    pub(crate) network_format: ConfigNetworkFormat,
    pub(crate) debug_extra: bool,
}

pub(crate) fn new_logger_settings() -> Result<LoggerSettings, String> {
    let config: LoggingConfiguration = match read_to_string(LOGGING_CONFIG_PATH) {
        Ok(raw) => {
            match toml::from_str(&raw) {
                Ok(config) => config,
                Err(error) => return Err(format!("toml::from_str failed: {:?}", error)),
            }
        }
        Err(error) => return Err(format!("read_to_string({:?}) failed: {:?}", LOGGING_CONFIG_PATH, error)),
    };

    return Ok(LoggerSettings {
        terminal_logs: config.terminal_logs,
        terminal_log_lvl: config.terminal_log_lvl,
        wasm_logging: false, // Must be false
        file_logs: config.file_logs,
        file_log_lvl: config.file_log_lvl,
        log_file_path: config.log_file_path,
        network_logs: config.network_logs,
        network_log_lvl: config.network_log_lvl,
        network_endpoint_url: config.network_endpoint_url,
        network_format: config.network_format.into(),
        debug_extra: config.debug_extra,
        async_logging: true, // Must be true
    });
}

#[derive(Deserialize)]
pub(crate) struct ReadConfiguration {
    pub(crate) this_server_url: String,
    pub(crate) cookie_name: String,
    pub(crate) cookie_domain: String,
    pub(crate) secret_cookie_hex_key: String,
    pub(crate) requesting_client_url: String,
    pub(crate) issuer_url: String,
    pub(crate) logout_url: String,
    pub(crate) client: String,
    pub(crate) client_secret: String,

    pub(crate) listen_address: Option<String>,
    pub(crate) listen_port: Option<u16>,
    pub(crate) workers: Option<usize>,
    pub(crate) redis_address: Option<String>,
    pub(crate) heartbeat_logging: Option<bool>,
    pub(crate) heartbeat_interval_hours: Option<u16>,
    pub(crate) machine_name: Option<String>,
    pub(crate) container_name: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) keep_alive_time_secs: Option<u64>,
    pub(crate) client_request_timeout_secs: Option<u64>,
    pub(crate) client_disconnect_timeout_secs: Option<u64>,
    pub(crate) max_connections: Option<usize>,
    pub(crate) early_refresh_skew_secs: Option<i64>,
    pub(crate) user_details_fail_when_not_authenticated: Option<bool>,
    pub(crate) default_username: Option<String>,
    pub(crate) default_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MainConfiguration {
    pub(crate) this_server_url: String,
    pub(crate) cookie_name: String,
    pub(crate) cookie_domain: String,
    pub(crate) secret_cookie_hex_key: String,
    pub(crate) requesting_client_url: String,
    pub(crate) issuer_url: String,
    pub(crate) logout_url: String,
    pub(crate) client: String,
    pub(crate) client_secret: String,

    pub(crate) listen_address: String,
    pub(crate) listen_port: u16,
    pub(crate) workers: usize,
    pub(crate) redis_address: String,
    pub(crate) heartbeat_logging: bool,
    pub(crate) heartbeat_interval_hours: u16,
    pub(crate) machine_name: String, 
    pub(crate) container_name: String,
    pub(crate) provider: String,
    pub(crate) keep_alive_time_secs: u64,
    pub(crate) client_request_timeout_secs: u64,
    pub(crate) client_disconnect_timeout_secs: u64,
    pub(crate) max_connections: usize,
    pub(crate) early_refresh_skew_secs: i64,
    pub(crate) user_details_fail_when_not_authenticated: bool,
    pub(crate) default_username: String,
    pub(crate) default_user_id: String,
}

impl MainConfiguration {
    pub(crate) fn new() -> Result<MainConfiguration, String> {
        let config: ReadConfiguration = match read_to_string(MAIN_CONFIG_PATH) {
            Ok(raw) => {
                match toml::from_str(&raw) {
                    Ok(config) => config,
                    Err(error) => return Err(format!("toml::from_str failed: {:?}", error)),
                }
            }
            Err(error) => return Err(format!("read_to_string({:?}) failed: {:?}", MAIN_CONFIG_PATH, error)),
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

        return Ok(MainConfiguration {
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
        });
    }
}

////////// END OF FILE //////////