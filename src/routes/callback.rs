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

use crate::OpenidClientData;
use crate::tools::error::Error;
use crate::tools::settings::MainConfiguration;
use better_logger::logger::debugx;
use std::collections::HashMap;
use actix_web::HttpResponse;
use actix_web::web::{Query, Data};
use actix_session::Session;
use openidconnect::{AuthorizationCode, OAuth2TokenResponse, TokenResponse, Nonce, PkceCodeVerifier};
use openidconnect::reqwest::ClientBuilder;
use openidconnect::reqwest::redirect::Policy;
use chrono::Utc;
use std::time::Duration as timeDuration;
use chrono::Duration as chronoDuration;

const HANDLER: &str = "callback"; // Used for error logging

pub(crate) async fn callback_handler(
    config_settings: Data<MainConfiguration>,  
    session: Session, 
    client_data: OpenidClientData,
    query: Query<HashMap<String, String>>, 
) -> HttpResponse {

    let rurl = &config_settings.requesting_client_url; // used for error logging

    // Check if the provider returned an error in the query string
    if let Some(error) = query.get("error") {
        let description = match query.get("error_description") {
            Some(desc) => desc,
            None => ""
        };
        return Error::send(session, rurl, HANDLER, &format!("oauth_error: {:?}", description), Error::fmt(error));
    }

    // Get the state returned in the query string
    let returned_state = match query.get("state") {
        Some(state) => state,
        None => return Error::send(session, rurl, HANDLER, "no state in query", None),
    };

    // Get the state that login_handler inserted into the session
    let extracted_state = match session.get::<String>("state") {
        Ok(option) => {
            match option {
                Some(state) => state,
                None => return Error::send(session, rurl, HANDLER, "no state in session", None),
            }
        }
        Err(error) => return Error::send(session, rurl, HANDLER, "extracted_state failed", Error::fmt(error)),
    };

    // The 2 state values must match
    if returned_state != &extracted_state {
        return Error::send(session, rurl, HANDLER, "state mismatch", None);
    }
    else {
        session.remove("state"); // No longer needed
    }

    let token_response = {
        let token_request = {
            let auth_code = match query.get("code") {
                Some(code) => code, // From query string
                None => return Error::send(session, rurl, HANDLER, "auth_code failed", None),
            };

            let pkce_verifier: String = match session.get("pkce_verifier") {
                Ok(verifier) => {
                    match verifier {
                        Some(ver) => ver, // login_handler inserted this into the session
                        None => return Error::send(session, rurl, HANDLER, "pkce_verifier is None", None),
                    }
                }
                Err(error) => return Error::send(session, rurl, HANDLER, "pkce_verifier failed", Error::fmt(error)),
            };

            match client_data.exchange_code(AuthorizationCode::new(auth_code.to_string())) { // build the token request
                Ok(request) => request.set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier)),
                Err(error) => return Error::send(session, rurl, HANDLER, "token_request failed", Error::fmt(error)),
            }
        }; 

        let http_client = match ClientBuilder::new().redirect(Policy::none()).timeout(timeDuration::from_secs(10)).build() {
            Ok(http_client) => http_client,
            Err(error) => return Error::send(session, rurl, HANDLER, "http_client failed", Error::fmt(error)),
        };

        match token_request.request_async(&http_client).await {
            Ok(response) => {
                session.remove("pkce_verifier"); // No longer needed
                response // Use the request and the http client to get the response
            }
            Err(error) => return Error::send(session, rurl, HANDLER, "token_response failed", Error::fmt(error)),
        }
    };

    // Get expiry
    if let Some(expires_in) = token_response.expires_in() {
        let expiry = match chronoDuration::from_std(expires_in) {
            Ok(time) => time,
            Err(error) => return Error::send(session, rurl, HANDLER, "expiry failed", Error::fmt(error)),
        };
        let new_expiry = (Utc::now() + expiry).timestamp();
 
        if let Err(error) = session.insert("token_expiry", new_expiry) {
            return Error::send(session, rurl, HANDLER, "failed to store token_expiry", Error::fmt(error));
        }
    } 
    else {
        return Error::send(session, rurl, HANDLER, "missing expiry", None);
    }

    // Add access token to the session
    if let Err(error) = session.insert("access_token", token_response.access_token().secret()) {
        return Error::send(session, rurl, HANDLER, "failed to store access_token", Error::fmt(error));
    }

    let refresh_token = match token_response.refresh_token() {
        Some(token) => token,
        None => return Error::send(session, rurl, HANDLER, "no refresh_token", None),
    };

    // Add refresh token to the session
    if let Err(error) = session.insert("refresh_token", refresh_token.secret()) {
        return Error::send(session, rurl, HANDLER, "failed to store refresh_token", Error::fmt(error));
    }

    let identification_token = match TokenResponse::id_token(&token_response) {
        Some(id_token) => id_token,
        None => return Error::send(session, rurl, HANDLER, "identification_token failed", None),
    };
    
    // Add id token to the session
    if let Err(error) = session.insert("id_token", identification_token.to_string()) {
        return Error::send(session, rurl, HANDLER, "failed to store id_token", Error::fmt(error));
    }

    // verified_claims is a strongly-typed claims object, a trusted set of user info
    let verified_claims = {
        let stored_nonce: String = match session.get("nonce") {
            Ok(option) => {
                match option {
                    Some(nonce) => nonce,
                    None => return Error::send(session, rurl, HANDLER, "no nonce", None),
                }
            }
            Err(error) => return Error::send(session, rurl, HANDLER, "stored_nonce failed", Error::fmt(error)),
        };

        let verifier = client_data.id_token_verifier();
        let nonce = Nonce::new(stored_nonce.clone()); 

        // The openidconnect crate uses the verifier and nonce to validate the claims
        match identification_token.claims(&verifier, &nonce) {
            Ok(claims) => claims,
            Err(error) => return Error::send(session, rurl, HANDLER, "id_token verification failed", Error::fmt(error)),
        }
    };

    session.remove("nonce"); // No longer needed

    let username = match verified_claims.preferred_username() {
        Some(name) => name.to_string(),
        None => return Error::send(session, rurl, HANDLER, "no preferred_username", None),
    };

    // Insert username to be used by user_details_handler
    if let Err(error) = session.insert("username", username) {
        return Error::send(session, rurl, HANDLER, "failed to store username", Error::fmt(error));
    }

    // Insert user id to be used by user_details_handler
    if let Err(error) = session.insert("user_id", verified_claims.subject().to_string()) {
        return Error::send(session, rurl, HANDLER, "failed to store user_id", Error::fmt(error));
    }

    // If all is good, send user to you web page
    debugx!("callback successful");
    return HttpResponse::Found()
    .insert_header(("Location", config_settings.requesting_client_url.as_str()))
    .insert_header(("Cache-Control", "no-store, no-cache, must-revalidate"))
    .insert_header(("Pragma", "no-cache"))
    .finish();
}

////////// END OF FILE //////////