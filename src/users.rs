use crate::{
    middleware::{fullpage_render::FullPageRender, login_guard::LoginGuard},
    models::user::User,
};
use actix_web::{get, web, HttpResponse, Responder};
use askama::Template;

pub fn users_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users").wrap(FullPageRender).service(get));
}

#[derive(Template)]
#[template(path = "users.html")]
pub struct UserListTemplate {
    pub users: Vec<User>,
}

#[get("/")]
async fn get(_: LoginGuard, db: web::Data<crate::DbPool>) -> impl Responder {
    let users = User::list(&db).await;
    let template = UserListTemplate { users };
    HttpResponse::Ok().body(template.render().unwrap())
}
