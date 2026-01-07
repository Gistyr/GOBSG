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

use crate::OpenidClientData;
use crate::tools::error::Error;
use crate::tools::settings::MainConfiguration;
use better_logger::logger::{debugx, error};
use actix_web::HttpResponse;
use actix_web::web::Data;
use actix_session::Session;
use openidconnect::{RefreshToken, OAuth2TokenResponse};
use chrono::Utc;
use serde_json::json;
use openidconnect::reqwest::ClientBuilder;
use openidconnect::reqwest::redirect::Policy;
use std::time::Duration as timeDuration;
use chrono::Duration as chronoDuration;

const HANDLER: &str = "sessionstatus"; // Used for error logging

pub(crate) async fn sessionstatus_handler(
    config_settings: Data<MainConfiguration>,
    session: Session,
    client_data: OpenidClientData,
) -> HttpResponse {

    let rurl = &config_settings.requesting_client_url; // used for error logging

    // No access token = not logged in
    let has_access_token = match session.get::<String>("access_token") {
        Ok(option) => {
            match option {
                Some(_) => true,
                None => return HttpResponse::Ok().json(json!({"status": "not_logged_in"})),
            }
        }
        Err(error) => return Error::send(session, rurl, HANDLER, "extracted_access_token failed", Error::fmt(error)),
    };

    // No refresh token = not logged in
    let refresh_token = {
        let extracted_refresh_token = match session.get::<String>("refresh_token") {
            Ok(option) => {
                match option {
                    Some(token) => token,
                    None => return HttpResponse::Ok().json(json!({"status": "not_logged_in"})),
                }
            }
            Err(error) => return Error::send(session, rurl, HANDLER, "extracted_refresh_token failed", Error::fmt(error)),
        };
        
        RefreshToken::new(extracted_refresh_token)
    };

    let extracted_access_token_expiry = match session.get::<i64>("token_expiry") {
        Ok(option) => {
            match option {
                Some(expiry) => expiry, // Time when the access token will expire
                None => return Error::send(session, rurl, HANDLER, "extracted_access_token failed", None),
            }
        }
        Err(error) => return Error::send(session, rurl, HANDLER, "extracted_access_token_expiry failed", Error::fmt(error)),
    };

    // If access token (minus the safety buffer) is not expired, user is logged in
    // Else use the refresh token to refresh the access token
    if Utc::now().timestamp() < extracted_access_token_expiry - config_settings.early_refresh_skew_secs {
        if has_access_token {
            debugx!("sessionstatus (1) successful");
            return actix_web::HttpResponse::Ok().json(json!({"status": "logged_in"}));
        }
        else { // Error because this condition should never happen
            error!("sessionstatus (1) failed");
            return HttpResponse::Ok().json(json!({"status": "not_logged_in"}));
        }
    } 
    else {
        // Use the refresh token to request a new access token
        // Depending on your token settings, a new refresh token may also be returned
        let token_response = {
            let refresh_token_request = match client_data.exchange_refresh_token(&refresh_token) {
                Ok(request) => request,
                Err(error) => return Error::send(session, rurl, HANDLER, "refresh_token_request failed", Error::fmt(error)),
            };

            let http_client = match ClientBuilder::new().redirect(Policy::none()).timeout(timeDuration::from_secs(10)).build() {
                Ok(http_client) => http_client,
                Err(error) => return Error::send(session, rurl, HANDLER, "http_client failed", Error::fmt(error)),
            };

            match refresh_token_request.request_async(&http_client).await {
                Ok(response) => response,
                Err(error) => return Error::send(session, rurl, HANDLER, "status token_response failed", Error::fmt(error)),
            }
        };

        // Calculate the absolute expiry time of the new access token, add to session
        let new_expiration = {
            if let Some(expires_in) = token_response.expires_in() { // "expires_in" represents the access token lifetime only, the refresh token's lifetime is managed by the provider and not returned here
                let expiry = match chronoDuration::from_std(expires_in) {
                    Ok(time) => time,
                    Err(error) => return Error::send(session, rurl, HANDLER, "expiry failed", Error::fmt(error)),
                };
                let new_expiry = (Utc::now() + expiry).timestamp();
        
                if let Err(error) = session.insert("token_expiry", new_expiry) {
                    return Error::send(session, rurl, HANDLER, "status failed to store token_expiry", Error::fmt(error));
                }

                new_expiry
            } 
            else {
                return Error::send(session, rurl, HANDLER, "status missing expiry", None);
            }
        };

        // Add new access token to session
        if let Err(error) = session.insert("access_token", token_response.access_token().secret()) {
            return Error::send(session, rurl, HANDLER, "status failed to store access_token", Error::fmt(error));
        }

        // If a new refresh token was returned, add it to the session
        if let Some(rtoken) = token_response.refresh_token() {
            if let Err(error) = session.insert("refresh_token", rtoken.secret()) {
                return Error::send(session, rurl, HANDLER, "status failed to store refresh_token", Error::fmt(error));
            }      
        }

        // The access token was successfully refreshed
        // If access token (minus the safety buffer) is not expired, user is logged in
        if Utc::now().timestamp() < new_expiration - config_settings.early_refresh_skew_secs {
            debugx!("sessionstatus (2) successful");
            return HttpResponse::Ok().json(json!({"status": "logged_in"}));
        }
        else {
            // After the refresh flow, the access token is still expired 
            error!("sessionstatus (2) failed");
            return HttpResponse::Ok().json(json!({"status": "not_logged_in"}));
        }
    }
}

////////// END OF FILE //////////