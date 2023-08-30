use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};

use crate::{
    middleware::{fullpage_render::FullPageRender, htmx::HtmxHeaders, login_guard::LoginGuard},
    models::user::{NewUser, Role, User},
    users::UserListTemplate,
};

pub fn user_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(
                web::resource("/{id}")
                    .route(web::delete().to(delete))
                    .route(web::put().to(put)),
            )
            .service(
                web::resource("/{id}/edit").route(web::get().to(get_edit).wrap(FullPageRender)),
            )
            .service(web::resource("/").route(web::post().to(post))),
    );
}

async fn delete(
    _: LoginGuard,
    req: HttpRequest,
    db: web::Data<crate::DbPool>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let deleted = User::delete(&mut conn, *id);
    if deleted {
        if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
            htmx.set_redirect("/users/");
        }
        HttpResponse::Ok().body("User deleted.")
    } else {
        HttpResponse::NotFound().body("User not found.")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct UserForm {
    name: String,
    password: String,
    email: Option<String>,
    role_id: Option<i32>,
}

impl From<UserForm> for NewUser {
    fn from(form: UserForm) -> Self {
        NewUser {
            name: form.name,
            password: form.password,
            email: form.email,
            role_id: form.role_id.unwrap_or(Role::User as i32),
        }
    }
}

#[derive(Template)]
#[template(path = "user_row.html")]
struct UserRowTemplate {
    user: User,
}

async fn post(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    form: web::Form<UserForm>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let new_user: NewUser = form.into_inner().into();
    let _ = new_user.insert(&mut conn);
    let users = User::list(&mut conn);
    let template = UserListTemplate { users };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[derive(Template)]
#[template(path = "user_edit.html")]
struct UserEditTemplate {
    user: User,
}

async fn get_edit(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let user = User::get(&mut conn, *id);
    if let Some(user) = user {
        let template = UserEditTemplate { user };
        HttpResponse::Ok().body(template.render().unwrap())
    } else {
        HttpResponse::NotFound().body("User not found")
    }
}

async fn put(
    _: LoginGuard,
    _req: HttpRequest,
    db: web::Data<crate::DbPool>,
    id: web::Path<i32>,
    form: web::Form<UserForm>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let user = User::get(&mut conn, *id);
    if let Some(mut user) = user {
        let form = form.into_inner();
        user.name = form.name;
        user.email = form.email;
        user.role_id = form.role_id.unwrap_or(Role::User as i32);
        let _ = User::update(&mut conn, *id, &user);
        HttpResponse::Ok().body("User successfully saved.")
    } else {
        HttpResponse::Ok().body("User to be updated not found.")
    }
}
