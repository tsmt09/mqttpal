use std::collections::HashMap;

use actix_session::{SessionMiddleware, storage::CookieSessionStore, Session, SessionStatus};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, cookie::Key, dev::Service, HttpRequest};
use actix_files::{Files, NamedFile};
use base64::Engine;
use futures_util::FutureExt;
use askama::Template;

mod middleware;
mod models;


#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    hx: bool
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    hx: bool
}

#[get("/")]
async fn index() -> impl Responder {
    let local_login = LoginTemplate { hx: false };
    HttpResponse::Ok().body(local_login.render().unwrap())
}

#[get("/dashboard")]
async fn dashboard(req: HttpRequest) -> impl Responder {
    let hx = req.headers().iter().any(|(name, val)| name.to_string().to_lowercase() == String::from("hx-request"));
    log::debug!("Is HX: {hx}");
    let template = DashboardTemplate { hx };
    HttpResponse::Ok().body(template.render().unwrap())
}


#[get("/login")]
async fn login(req: HttpRequest) -> impl Responder {
    let hx = req.headers().iter().any(|(name, val)| name.to_string().to_lowercase() == String::from("hx-request"));
    log::debug!("Is HX: {hx}");
    let template = LoginTemplate { hx };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[post("/login")]
    async fn login_post(form: web::Form<models::user::UserForm>, session: Session) -> impl Responder {
    log::debug!("User: {form:?}");
    log::debug!("Session status: {:?}", session.status());
    HttpResponse::Unauthorized().append_header(("HX-Retarget", "#form-errors")).append_header(("HX-Reswap", "innerHTML")).body("<mark>Password wrong or user unknown.</mark>") 
    //HttpResponse::Ok().append_header(("HX-Redirect", "/")).finish()
}

#[get("/favicon.ico")]
async fn favicon(session: Session) -> impl Responder {
    NamedFile::open_async("static/favicon.ico").await
}

fn create_session_key() -> Key {
    let key = Key::generate();
    let key_sign = base64::engine::general_purpose::STANDARD.encode(key.signing());
    let key_master = base64::engine::general_purpose::STANDARD.encode(key.master());
    println!("sign: {}", key_sign);
    println!("master: {}", key_master);
    key
}

fn get_session_key() -> Key {
    let key = std::env::var("SESSION_KEY").expect("cannot get SESSION_KEY from ENV");
    Key::from(&base64::engine::general_purpose::STANDARD.decode(key).expect("cannot decode base64 SESSION_KEY"))
}

#[actix_web::main]
async fn main () -> std::io::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init_timed();
    log::info!("Booting MQTTPal");
    HttpServer::new(|| {
        App::new()
        .wrap(actix_web::middleware::Logger::default())
        .wrap(
            SessionMiddleware::new(CookieSessionStore::default(), get_session_key())
        )
        // resources which are always available
        .service(actix_files::Files::new("/css/", "static/css/"))
        .service(actix_files::Files::new("/js/", "static/js/"))
        .service(actix_files::Files::new("/svg/", "static/svg/"))
        .service(login)
        .service(login_post)
        .service(favicon)
        .service(index)
        .service(dashboard)
        // guarded resources
        /* .service(
            web::route("/")
            .wrap_fn(|req, srv| {
                log::info!("Guard Middleware enter");
                srv.call(req).map(|res| {
                    log::info!("Guard Middleware exit");
                    res
                })
            })
        )*/
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
