use actix_session::Session;
use actix_web::{
    dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpMessage, HttpRequest,
    HttpResponse,
};
use futures_util::future::Ready;

pub struct Authorized;

impl FromRequest for Authorized {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Ok(session) = Session::extract(req).into_inner() {
            if let Ok(Some(loggedin)) = session.get::<String>("loggedin") {
                if loggedin == "true" {
                    return futures_util::future::ready(Ok(Authorized));
                }
            }
            return futures_util::future::ready(Err(ErrorUnauthorized("user not logged in")));
        }
        futures_util::future::ready(Err(ErrorUnauthorized("session not available")))
    }
}
