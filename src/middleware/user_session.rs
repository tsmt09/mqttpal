use actix_session::Session;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures_util::future::Ready;

#[derive(Debug, Clone)]
pub struct UserSession {
    pub username: Option<String>,
}

impl FromRequest for UserSession {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Ok(session) = Session::extract(req).into_inner() {
            return futures_util::future::ready(Ok(UserSession {
                username: session.get::<String>("username").unwrap_or(None),
            }));
        }
        futures_util::future::ready(Ok(UserSession { username: None }))
    }
}
