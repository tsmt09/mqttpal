use crate::{
    middleware::{htmx::HtmxHeaders, login_guard::LoginGuard, user_session::UserSession},
    models::user::User,
};
use actix_session::Session;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct LoginForm {
    name: String,
    password: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub hx: bool,
    pub user: Option<String>,
}

#[get("/")]
async fn get(req: HttpRequest, usersession: UserSession) -> impl Responder {
    let template = if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        log::debug!("Is htmx req? {}", htmx.request());
        if htmx.request() {
            log::debug!("Set redirect!");
            htmx.set_push_url("/login/");
        }
        LoginTemplate {
            hx: htmx.request(),
            user: usersession.username,
        }
    } else {
        LoginTemplate {
            hx: false,
            user: usersession.username,
        }
    };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[post("/logout/")]
async fn logout(_: LoginGuard, req: HttpRequest, session: Session) -> impl Responder {
    session.purge();
    if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        if htmx.request() {
            htmx.set_redirect("/login/");
            return HttpResponse::Ok().finish();
        }
    }
    HttpResponse::Ok().finish()
}

#[post("/login/")]
async fn post(
    req: HttpRequest,
    db: web::Data<crate::DbPool>,
    form: web::Form<LoginForm>,
    session: Session,
) -> impl Responder {
    log::debug!("User: {form:?}");
    let mut conn = db.get().expect("no connection available");
    let is_user = User::check(&mut conn, &form.name, &form.password);
    if is_user {
        let _ = session.insert("loggedin", "true");
        let _ = session.insert("username", &form.name);
    } else {
        session.purge();
    }
    log::debug!("Session status: {:?}", session.status());
    // handle differently based on htmx or not
    if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        if htmx.request() {
            if is_user {
                htmx.set_redirect("/");
                HttpResponse::Ok().finish()
            } else {
                htmx.set_retarget("#form-errors");
                htmx.set_reswap("innerHtml");
                HttpResponse::Ok().body("<mark>Password wrong or user unknown.</mark>")
            }
        } else {
            HttpResponse::Unauthorized().finish()
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}
