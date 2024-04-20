use actix_session::Session;
use actix_web::{dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use futures_util::future::Ready;

pub struct LoginGuard;

impl FromRequest for LoginGuard {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Ok(session) = Session::extract(req).into_inner() {
            log::debug!("{:?}", session.entries());
            if let Ok(Some(loggedin)) = session.get::<String>("loggedin") {
                if loggedin == "true" {
                    return futures_util::future::ready(Ok(LoginGuard));
                }
            }
            return futures_util::future::ready(Err(ErrorUnauthorized("user not logged in")));
        }
        futures_util::future::ready(Err(ErrorUnauthorized("No session")))
    }
}
