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

pub(crate) mod routes;
pub(crate) mod tools;

use routes::login::login_handler;
use routes::callback::callback_handler;
use routes::sessionstatus::sessionstatus_handler;
use routes::details::user_details_handler;
use routes::logout::logout_handler;
use crate::tools::settings::{new_logger_settings, MainConfiguration};
use better_logger::logger;
use std::sync::{Arc, Mutex};
use std::process::exit;
use actix_web::{HttpServer, App, web};
use actix_web::web::Data;
use actix_web::middleware::DefaultHeaders;
use actix_web::cookie::{SameSite, Key};
use actix_web::http::header::{AUTHORIZATION, ACCEPT, CONTENT_TYPE};
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_session::config::PersistentSession;
use actix_cors::Cors;
use openidconnect::{IssuerUrl, ClientId, ClientSecret, RedirectUrl};
use openidconnect::core::{CoreProviderMetadata, CoreClient};
use openidconnect::reqwest::ClientBuilder;
use openidconnect::reqwest::redirect::Policy;
use hex::FromHex;
use redis::Client;
use tokio::sync::Notify;
use tokio::time::{interval, MissedTickBehavior};
use std::time::Duration as timeDuration;
use actix_web::cookie::time::Duration as cookieTimeDuration;
use tokio::time::Duration as tokioDuration;

pub(crate) type OpenidClientData = actix_web::web::Data<
    openidconnect::Client<openidconnect::EmptyAdditionalClaims, 
    openidconnect::core::CoreAuthDisplay, 
    openidconnect::core::CoreGenderClaim, 
    openidconnect::core::CoreJweContentEncryptionAlgorithm, 
    openidconnect::core::CoreJsonWebKey, 
    openidconnect::core::CoreAuthPrompt, 
    openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>, 
    openidconnect::StandardTokenResponse<openidconnect::IdTokenFields<openidconnect::EmptyAdditionalClaims, 
    openidconnect::EmptyExtraTokenFields, 
    openidconnect::core::CoreGenderClaim, 
    openidconnect::core::CoreJweContentEncryptionAlgorithm, 
    openidconnect::core::CoreJwsSigningAlgorithm>, 
    openidconnect::core::CoreTokenType>, 
    openidconnect::StandardTokenIntrospectionResponse<openidconnect::EmptyExtraTokenFields, 
    openidconnect::core::CoreTokenType>, 
    openidconnect::core::CoreRevocableToken, 
    openidconnect::StandardErrorResponse<openidconnect::RevocationErrorResponseType>, 
    openidconnect::EndpointSet, 
    openidconnect::EndpointNotSet, 
    openidconnect::EndpointNotSet, 
    openidconnect::EndpointNotSet, 
    openidconnect::EndpointMaybeSet, 
    openidconnect::EndpointMaybeSet>
>;

#[actix_web::main]
async fn main() {
    if let Err(error) = logger::init(
        match new_logger_settings() {
            Ok(settings) => settings,
            Err(error) => {
                logger::error!("{:?}", error);
                exit(1);
            }
        }
    ) {
        eprintln!("{:?}", error);
        std::process::exit(1);
    }

    let configuration_settings = match MainConfiguration::new() {
        Ok(settings) => settings,
        Err(error) => {
            logger::error!("{:?}", error);
            exit(1);
        }
    };

    let machine_name_1 = configuration_settings.machine_name.clone();
    let container_name_1 = configuration_settings.container_name.clone();
    let provider_1 = configuration_settings.provider.clone();
    let machine_name_2 = configuration_settings.machine_name.clone();
    let container_name_2 = configuration_settings.container_name.clone();
    let provider_2 = configuration_settings.provider.clone();
    
    let shutdown = Arc::new(Notify::new());
    if configuration_settings.heartbeat_logging {        
        let shutdown_for_task = shutdown.clone();
        tokio::spawn(async move {
            let mut time = interval(tokioDuration::from_secs(configuration_settings.heartbeat_interval_hours as u64 * 60 * 60));
            time.set_missed_tick_behavior(MissedTickBehavior::Delay);
            time.tick().await;
            loop {
                tokio::select! {
                    _ = time.tick() => {
                        let heartbeat = format!("\n{} - {} - {}: {}", 
                            machine_name_2, 
                            container_name_2,
                            provider_2,
                            "HEARTBEAT --- HEARTBEAT"
                        );   
                        logger::info!("{}", heartbeat);
                    }
                    _ = shutdown_for_task.notified() => break,
                }
            }
        });
    }

    let starting_log_message = format!("\n{} - {} - {}: {}", 
        machine_name_1, 
        container_name_1,
        provider_1,
        "STARTING"
    );
    logger::info!("{}", starting_log_message);

    match init(configuration_settings).await {
        Ok(_) => {
            shutdown.notify_waiters();
            let log_message_1 = format!("\n{} - {}: {}\n{}", 
                machine_name_1, 
                container_name_1,
                "EXITED WITH CONDITION: \"Ok()\"",
                "If this was not planned, is an error"
            );
            logger::warn!("{}", log_message_1);
        }
        Err(error) => {
            shutdown.notify_waiters();
            let log_message_2 = format!("\n{} - {}: {}\n{}\n{}", 
                machine_name_1, 
                container_name_1,
                "EXITED WITH CONDITION: \"Err()\"",
                "ERROR:",
                error
            );
            logger::error!("{}", log_message_2);
        }
    }
}

pub(crate) async fn init(config_settings: MainConfiguration) -> Result<(), String> {
    let config_settings_data = Data::new(config_settings);

    let openid_client_data = {
        let issuer_url = match IssuerUrl::new(config_settings_data.issuer_url.as_str().to_string()) {
            Ok(url) => url,
            Err(error) => return Err(format!("{:?}", error)),
        };

        let http_client = match ClientBuilder::new().redirect(Policy::none()).timeout(timeDuration::from_secs(10)).build() {
            Ok(http_client) => http_client,
            Err(error) => return Err(format!("{:?}", error)),
        };

        let provider_metadata = match CoreProviderMetadata::discover_async(issuer_url, &http_client).await {
            Ok(data) => data,
            Err(error) => return Err(format!("{:?}", error)),
        };

        let redirect_url = match RedirectUrl::new(format!("{}/callback", config_settings_data.this_server_url)) {
            Ok(url) => url,
            Err(error) => return Err(format!("{:?}", error)),
        };

        let openid_client = {
            CoreClient::from_provider_metadata(
                provider_metadata, 
                ClientId::new(config_settings_data.client.as_str().to_string()), 
                Some(ClientSecret::new(config_settings_data.client_secret.as_str().to_string()))
            ).set_redirect_uri(redirect_url)
        }; 

        Data::new(openid_client)
    };

    let cookie_key = {
        let key_bytes = match <[u8; 64]>::from_hex(config_settings_data.secret_cookie_hex_key.as_str()) {
            Ok(bytes) => bytes,
            Err(error) => return Err(format!("{:?}", error)),
        };

        Key::from(&key_bytes)
    };

    let redis_store = match RedisSessionStore::new(config_settings_data.redis_address.as_str()).await {
        Ok(store) => store,
        Err(error) => return Err(format!("{:?}", error)),
    };

    let wrapped_redis_client = {
        let redis_client = match Client::open(config_settings_data.redis_address.as_str()) {
            Ok(client) => client,
            Err(error) => return Err(format!("{:?}", error)),
        };

        Data::new(Mutex::new(redis_client))
    };

    let requesting_client_url = config_settings_data.requesting_client_url.clone();
    let cookie_name = config_settings_data.cookie_name.clone();
    let cookie_domain = config_settings_data.cookie_domain.clone();
    let listen_address = config_settings_data.listen_address.clone();
    let listen_port = config_settings_data.listen_port;
    let workers = config_settings_data.workers;
    let keep_alive = config_settings_data.keep_alive_time_secs as u64;
    let client_request_timeout = config_settings_data.client_request_timeout_secs as u64;
    let client_disconnect_timeout = config_settings_data.client_disconnect_timeout_secs as u64;
    let max_connections = config_settings_data.max_connections as usize;

    match HttpServer::new(
        move || {App::new()
            .app_data(config_settings_data.clone())
            .app_data(openid_client_data.clone())
            .app_data(wrapped_redis_client.clone())
            .wrap(Cors::default()
                .allowed_origin(requesting_client_url.as_str())
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                .supports_credentials()
            )
            .wrap(SessionMiddleware::builder(redis_store.clone(), cookie_key.clone(),)
                .cookie_name(cookie_name.clone())
                .cookie_domain(Some(cookie_domain.clone()))
                .cookie_secure(true).cookie_http_only(true).cookie_same_site(SameSite::None)
                .session_lifecycle(PersistentSession::default().session_ttl(cookieTimeDuration::days(7)))
                .build(),
            )
            .wrap(DefaultHeaders::new()
                .add(("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload"))
                .add(("X-Frame-Options", "DENY"))
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("Referrer-Policy", "no-referrer")),
            )
            .route("/login", web::get().to(login_handler))
            .route("/callback", web::get().to(callback_handler))
            .route("/sessionstatus", web::get().to(sessionstatus_handler))
            .route("/details", web::get().to(user_details_handler))
            .route("/logout", web::get().to(logout_handler))
        }
    )
    .workers(workers)
    .keep_alive(timeDuration::from_secs(keep_alive))
    .client_request_timeout(timeDuration::from_secs(client_request_timeout))
    .client_disconnect_timeout(timeDuration::from_secs(client_disconnect_timeout))
    .max_connections(max_connections)
    .bind((listen_address.as_str(), listen_port)) {
        Ok(server) => {
            match server.run().await {
                Ok(_) =>return Ok(()),
                Err(error) => return Err(format!("{:?}", error)),
            }
        }
        Err(error) => return Err(format!("{:?}", error)),
    }
}    

////////// END OF FILE //////////