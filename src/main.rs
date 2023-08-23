use crate::middleware::fullpage_render::FullPageRender;
use crate::middleware::user_session::UserSession;
use crate::models::mqtt_client::NewMqttClient;
use crate::models::user::Role;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, get, web, App, HttpResponse, HttpServer, Responder};
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
mod mqtt;
mod mqtt_clients;
pub mod schema;
mod user;
mod users;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[derive(Subcommand, Debug)]
enum CliCommands {
    Serve,
    CreateSessionKey,
    CreateInitUser(CreateInitUserArgs),
    CreateClient(CreateClientArgs),
}

#[derive(Args, Debug)]
struct CreateInitUserArgs {
    name: String,
    password: String,
    email: Option<String>,
    role_id: Option<i32>,
}

#[derive(Args, Debug)]
struct CreateClientArgs {
    name: String,
    url: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct CliArgs {
    // command used (default "server")
    #[command(subcommand)]
    command: CliCommands,
}

#[get("/")]
async fn index(usession: UserSession) -> impl Responder {
    if usession.username.is_some() {
        return HttpResponse::Ok().body("<h1> TODO </h1>");
    }
    let local_login = login::LoginTemplate {
        hx: false,
        user: usession.username,
    };
    HttpResponse::Ok().body(local_login.render().unwrap())
}

#[get("/favicon.ico")]
async fn favicon(_session: Session) -> impl Responder {
    NamedFile::open_async("static/favicon.ico").await
}

fn create_session_key() -> Key {
    Key::generate()
}

fn get_session_key() -> Key {
    let key = std::env::var("SESSION_KEY").unwrap_or_else(|_| {
        let key = create_session_key();
        let key_master = base64::engine::general_purpose::STANDARD.encode(key.master());
        let key_sign = base64::engine::general_purpose::STANDARD.encode(key.signing());
        log::info!("No session key ENV found, generating new one. Please set SESSION_KEY to the following value before restarting container: {}", key_master);
        log::info!("Signing key: {}", key_sign);
        key_master
    });
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
            log::info!("Generating session key");
            let key = create_session_key();
            let key_sign = base64::engine::general_purpose::STANDARD.encode(key.signing());
            let key_master = base64::engine::general_purpose::STANDARD.encode(key.master());
            log::info!("sign: {}", key_sign);
            log::info!("master: {}", key_master);
            Ok(())
        }
        CliCommands::CreateInitUser(user) => {
            let mut conn = pool.get().expect("cannot get connection from pool!");
            let existing_users = models::user::User::list(&mut conn).len();
            if existing_users > 0 {
                log::info!("Users already exist, skipping init user creation");
                return Ok(());
            }
            let user = models::user::NewUser {
                name: user.name,
                password: user.password,
                email: user.email,
                role_id: user.role_id.unwrap_or(Role::Admin as i32),
            };
            let result = user.insert(&mut conn);
            log::info!("Inserted User: {result:?}");
            // do some serve
            Ok(())
        }
        CliCommands::CreateClient(client) => {
            let new_client = NewMqttClient {
                name: client.name,
                url: client.url,
            };
            let mut conn = pool.get().expect("cannot get connection from pool!");
            let result = new_client.insert(&mut conn);
            log::info!("Inserted Client: {result:?}");
            // do some serve
            Ok(())
        }
        CliCommands::Serve => {
            let mut mqtt_manager = mqtt::MqttClientManager::new();
            let session_key = get_session_key();
            let clients = models::mqtt_client::MqttClient::list(&mut pool.get().unwrap());
            for client in clients {
                mqtt_manager
                    .register_client(client.name, client.url)
                    .await
                    .unwrap();
            }
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(pool.clone()))
                    .app_data(web::Data::new(mqtt_manager.clone()))
                    .wrap(actix_web::middleware::Logger::default())
                    .wrap(SessionMiddleware::new(
                        CookieSessionStore::default(),
                        session_key.clone(),
                    ))
                    .wrap(Htmx)
                    // resources which are always available
                    .service(actix_files::Files::new("/css/", "static/css/"))
                    .service(actix_files::Files::new("/js/", "static/js/"))
                    .service(login::post)
                    .service(login::logout)
                    .service(user::delete_user)
                    .service(user::post)
                    .service(favicon)
                    .service(
                        web::scope("/users")
                            .wrap(FullPageRender)
                            .service(users::get),
                    )
                    .service(
                        web::scope("/mqtt_clients")
                            .wrap(FullPageRender)
                            .service(mqtt_clients::get),
                    )
                    .service(
                        web::scope("/login")
                            .wrap(FullPageRender)
                            .service(login::get),
                    )
                    .service(web::scope("/").wrap(FullPageRender).service(index))
                // guarded resources
            })
            .bind(("0.0.0.0", 8080))?
            .run()
            .await
        }
    }
}