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
use crate::error::Error;
use settings::MainConfiguration;
use better_logger::logger::debugx;
use actix_web::HttpResponse;
use actix_web::web::Data;
use actix_session::Session;
use openidconnect::{CsrfToken, Nonce, Scope, PkceCodeChallenge};
use openidconnect::core::CoreAuthenticationFlow;

const HANDLER: &str = "login"; // Used for error logging

pub(crate) async fn login_handler(
    config_settings: Data<MainConfiguration>, 
    session: Session, 
    client_data: OpenidClientData,
) -> HttpResponse {
    
    let rurl = &config_settings.requesting_client_url; // used for error logging

    // Create and insert into the session, callback_handler will validate pkce_verifier
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    if let Err(error) = session.insert("pkce_verifier", pkce_verifier.secret()) {
        return Error::send(session, rurl, HANDLER, "failed to store pkce_verifier", Error::fmt(error));
    }

    // Use the openidconnect crate to build these items
    let (auth_url, csrf_token, nonce) = {
        client_data
        .authorize_url(CoreAuthenticationFlow::AuthorizationCode, CsrfToken::new_random, Nonce::new_random,)
        .set_pkce_challenge(pkce_challenge)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .add_scope(Scope::new("groups".to_string()))
        .url()
    }; 
    
    // Insert state into the session, callback_handler will validate this value
    if let Err(error) = session.insert("state", csrf_token.secret()) {
        return Error::send(session, rurl, HANDLER, "failed to store state", Error::fmt(error));
    }
    
    // Insert nonce into the session, callback_handler will validate this value
    if let Err(error) = session.insert("nonce", nonce.secret()) {
        return Error::send(session, rurl, HANDLER, "failed to store nonce", Error::fmt(error));
    }
    
    // If all is good, send user to your login page
    debugx!("login successful");
    return HttpResponse::Found()
    .insert_header(("Location", auth_url.as_str()))
    .insert_header(("Cache-Control", "no-store, no-cache, must-revalidate"))
    .insert_header(("Pragma", "no-cache"))
    .finish();
}

////////// END OF FILE //////////