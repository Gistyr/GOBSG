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

use crate::tools::error::Error;
use crate::tools::settings::MainConfiguration;
use better_logger::logger::debugx;
use actix_web::HttpResponse;
use actix_web::web::Data;
use actix_session::Session;
use url::Url;

const HANDLER: &str = "logout"; // Used for error logging

pub(crate) async fn logout_handler(
    config_settings: Data<MainConfiguration>,
    session: Session,
) -> HttpResponse {

    let rurl = &config_settings.requesting_client_url; // used for error logging

    let extracted_id_token = match session.get::<String>("id_token") {
        Ok(option) => {
            match option {
                Some(token) => token,
                None => return Error::send(session, rurl, HANDLER, "no id token", None),
            }
        }
        Err(error) => return Error::send(session, rurl, HANDLER, "extracted_id_token failed", Error::fmt(error)),
    };

    let logout_url = {
        let mut url = match Url::parse(&config_settings.logout_url) {
            Ok(url) => url,
            Err(error) => return Error::send(session, rurl, HANDLER, "bad end session url", Error::fmt(error)),
        };
        url.query_pairs_mut().append_pair("id_token_hint", &extracted_id_token).append_pair("post_logout_redirect_uri", config_settings.requesting_client_url.as_str());
        url
    };

    session.purge();    
    debugx!("logout successful");
    return HttpResponse::Found()
    .insert_header(("Location", logout_url.as_str()))
    .insert_header(("Cache-Control", "no-store, no-cache, must-revalidate"))
    .insert_header(("Pragma", "no-cache"))
    .finish();
}

////////// END OF FILE //////////