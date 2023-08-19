use crate::middleware::htmx::HtmxHeaders;
use crate::middleware::login_guard::LoginGuard;
use crate::middleware::user_session::UserSession;
use crate::models::user::Role;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key, get, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use base64::Engine;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use diesel::SqliteConnection;
use middleware::htmx::Htmx;

mod login;
mod middleware;
mod models;
pub mod schema;
mod user;
mod users;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[derive(Subcommand, Debug)]
enum CliCommands {
    Serve,
    CreateSessionKey,
    CreateUser(CreateUserArgs),
}

#[derive(Args, Debug)]
struct CreateUserArgs {
    name: String,
    password: String,
    email: Option<String>,
    role_id: Option<i32>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct CliArgs {
    // command used (default "server")
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    hx: bool,
    user: Option<String>,
}

#[get("/")]
async fn index(usession: UserSession) -> impl Responder {
    if usession.username.is_some() {
        let template = DashboardTemplate {
            hx: false,
            user: usession.username,
        };
        return HttpResponse::Ok().body(template.render().unwrap());
    }
    let local_login = login::LoginTemplate {
        hx: false,
        user: usession.username,
    };
    HttpResponse::Ok().body(local_login.render().unwrap())
}

#[get("/")]
async fn dashboard(req: HttpRequest, _: LoginGuard, usession: UserSession) -> impl Responder {
    let template = if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        log::debug!("Is htmx req? {}", htmx.request());
        if htmx.request() {
            log::debug!("Set redirect!");
            htmx.set_push_url("/dashboard/");
        }
        DashboardTemplate {
            hx: htmx.request(),
            user: usession.username,
        }
    } else {
        DashboardTemplate {
            hx: false,
            user: usession.username,
        }
    };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[get("/favicon.ico")]
async fn favicon(_session: Session) -> impl Responder {
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
    Key::from(
        &base64::engine::general_purpose::STANDARD
            .decode(key)
            .expect("cannot decode base64 SESSION_KEY"),
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init_timed();
    let cli = CliArgs::parse();
    log::debug!("Command Line Args: {:?}", cli);
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = diesel::r2d2::ConnectionManager::<SqliteConnection>::new(&database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("failed to build connection pool");
    match cli.command {
        CliCommands::CreateSessionKey => {
            create_session_key();
            Ok(())
        }
        CliCommands::CreateUser(user) => {
            let user = models::user::NewUser {
                name: user.name,
                password: user.password,
                email: user.email,
                role_id: user.role_id.unwrap_or(Role::Admin as i32),
            };
            let mut conn = pool.get().expect("cannot get connection from pool!");
            let result = user.insert(&mut conn);
            log::info!("Inserted User: {result:?}");
            // do some serve
            Ok(())
        }
        CliCommands::Serve => {
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(pool.clone()))
                    .wrap(actix_web::middleware::Logger::default())
                    .wrap(SessionMiddleware::new(
                        CookieSessionStore::default(),
                        get_session_key(),
                    ))
                    .wrap(Htmx)
                    .service(
                        web::scope("/dashboard")
                            .wrap(SessionMiddleware::new(
                                CookieSessionStore::default(),
                                get_session_key(),
                            ))
                            .service(dashboard),
                    )
                    // resources which are always available
                    .service(actix_files::Files::new("/css/", "static/css/"))
                    .service(actix_files::Files::new("/js/", "static/js/"))
                    .service(actix_files::Files::new("/svg/", "static/svg/"))
                    .service(login::login)
                    .service(login::login_post)
                    .service(login::logout)
                    .service(users::get)
                    .service(user::delete_user)
                    .service(user::post)
                    .service(favicon)
                    .service(index)
                // guarded resources
            })
            .bind(("0.0.0.0", 8081))?
            .run()
            .await
        }
    }
}
