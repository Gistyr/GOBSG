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

use crate::tools::error::Error;
use crate::tools::settings::MainConfiguration;
use better_logger::logger::debugx;
use actix_web::HttpResponse;
use actix_web::web::Data;
use actix_session::Session;
use serde::Serialize;

const HANDLER: &str = "details"; // Used for error logging

#[derive(Debug, Serialize)]
struct UserDetails {
    username: String,
    user_id: String,
}

pub(crate) async fn user_details_handler(
    config_settings: Data<MainConfiguration>, 
    session: Session
) -> HttpResponse {

    let rurl = &config_settings.requesting_client_url; // used for error logging

    let default_user_details = UserDetails { 
        username: config_settings.default_username.clone(),
        user_id: config_settings.default_user_id.clone(),
    };

    // Determine what to do based on username in session
    let extracted_username = match session.get::<String>("username") {
        Ok(option) => {
            match option {
                Some(name) => name,
                None => {
                    if config_settings.user_details_fail_when_not_authenticated {
                        return Error::send(session, rurl, HANDLER, "no username in session", None)
                    } else {
                        return HttpResponse::Ok().json(default_user_details);
                    }
                }
            }
        }
        Err(error) => return Error::send(session, rurl, HANDLER, "extracted_username failed", Error::fmt(error)),
    };

    // Determine what to do based on user_id in session
    let extracted_user_id = match session.get::<String>("user_id") {
        Ok(option) => {
            match option {
                Some(id) => id,
                None => {
                    if config_settings.user_details_fail_when_not_authenticated {
                        return Error::send(session, rurl, HANDLER, "no user_id in session", None)
                    } else {
                        return HttpResponse::Ok().json(default_user_details);
                    }
                }
            }
        }
        Err(error) => return Error::send(session, rurl, HANDLER, "extracted_user_id failed", Error::fmt(error)),
    };

    let user_details = UserDetails { 
        username: extracted_username,
        user_id: extracted_user_id,
    };

    // If all is good, return user details
    debugx!("user_details successful");
    return HttpResponse::Ok().json(user_details);
}

////////// END OF FILE //////////