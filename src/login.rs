use crate::{
    middleware::{
        fullpage_render::FullPageRender, htmx::HtmxHeaders, login_guard::LoginGuard,
        user_session::UserSession,
    },
    models::user::User,
    oauth::OauthConfigs,
};
use actix_session::Session;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};

pub fn login_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/login").service(
            web::resource("/")
                .route(web::get().to(get).wrap(FullPageRender))
                .route(web::post().to(post)),
        ),
    );
    cfg.service(web::scope("/logout").service(web::resource("/").route(web::post().to(logout))));
}

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
    pub configs: Vec<(String, String)>,
}

async fn get(req: HttpRequest, usersession: UserSession, configs: OauthConfigs) -> impl Responder {
    let configs: Vec<(String, String)> = configs
        .iter()
        .map(|(k, v)| (k.clone(), v.ui_name.clone()))
        .collect();
    let template = if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        log::debug!("Is htmx req? {}", htmx.request());
        if htmx.request() {
            log::debug!("Set redirect!");
            htmx.set_push_url("/login/");
        }
        LoginTemplate {
            hx: htmx.request(),
            user: usersession.username,
            configs,
        }
    } else {
        LoginTemplate {
            hx: false,
            user: usersession.username,
            configs,
        }
    };
    HttpResponse::Ok().body(template.render().unwrap())
}

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

async fn post(
    req: HttpRequest,
    db: web::Data<crate::DbPool>,
    form: web::Form<LoginForm>,
    session: Session,
) -> impl Responder {
    log::debug!("User: {form:?}");
    let is_user = User::check(
        &db,
        &form.name,
        &form.password,
        crate::models::user::UserSource::Local,
    )
    .await;
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
