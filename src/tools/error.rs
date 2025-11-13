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

use std::fmt::Debug;
use actix_web::HttpResponse;
use actix_session::Session;
use better_logger::logger::error;

#[derive(Debug)]
pub(crate) struct Error;
impl Error {
    #[inline]
    pub(crate) fn fmt(err: impl Debug) -> Option<String> {
        return Some(format!("{:?}", err));
    }

    // log, purge, redirect the user
    pub(crate) fn send(sess: Session, redirect_url: &str, handler: &str, msg: &str, err: Option<String>) -> HttpResponse {
        let error_message = match err {
            Some(error) => format!("({}) {}: {}", handler, msg, error),
            None => format!("({}) {}", handler, msg),
        };

        error!("{}", error_message);
        sess.purge();
        return HttpResponse::Found()
        .insert_header(("Location", redirect_url))
        .insert_header(("Cache-Control", "no-store, no-cache, must-revalidate"))
        .insert_header(("Pragma", "no-cache"))
        .finish();
    }
}

////////// END OF FILE //////////