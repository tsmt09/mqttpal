use actix_web::{get, web, HttpResponse, Responder};
use askama::Template;

use crate::{
    middleware::login_guard::LoginGuard,
    models::user::{Role, User},
};

#[derive(Template)]
#[template(path = "users.html")]
pub struct UserListTemplate {
    pub users: Vec<User>,
}

#[get("/")]
async fn get(_: LoginGuard, db: web::Data<crate::DbPool>) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let users = User::list(&mut conn);
    let template = UserListTemplate { users };
    HttpResponse::Ok().body(template.render().unwrap())
}
