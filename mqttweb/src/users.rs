use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use askama::Template;

use crate::{
    middleware::{htmx::HtmxHeaders, login_guard::LoginGuard, user_session::UserSession},
    models::user::User,
};

#[derive(Template)]
#[template(path = "users.html")]
pub struct UserListTemplate {
    hx: bool,
    user: Option<String>,
    pub users: Vec<User>,
}

#[get("/users/")]
async fn get(
    _: LoginGuard,
    usession: UserSession,
    req: HttpRequest,
    db: web::Data<crate::DbPool>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let users = User::list(&mut conn);
    let template = if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        log::debug!("Is htmx req? {}", htmx.request());
        if htmx.request() {
            log::debug!("Set redirect!");
            htmx.set_push_url("/users/");
        }
        UserListTemplate {
            hx: htmx.request(),
            user: usession.username,
            users,
        }
    } else {
        UserListTemplate {
            hx: false,
            user: usession.username,
            users,
        }
    };
    HttpResponse::Ok().body(template.render().unwrap())
}
