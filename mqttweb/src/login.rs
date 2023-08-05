use crate::{middleware::htmx::HtmxHeaders, models::user::User};
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
}

#[get("/login/")]
async fn login(req: HttpRequest) -> impl Responder {
    let template = if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        log::debug!("Is htmx req? {}", htmx.request());
        if htmx.request() {
            log::debug!("Set redirect!");
            htmx.set_push_url("/login/");
        }
        LoginTemplate { hx: htmx.request() }
    } else {
        LoginTemplate { hx: false }
    };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[post("/login/")]
async fn login_post(
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
    } else {
        session.purge();
    }
    log::debug!("Session status: {:?}", session.status());
    // handle differently based on htmx or not
    if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        if htmx.request() {
            if is_user {
                htmx.set_redirect("/dashboard/");
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
