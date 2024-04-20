use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};

use crate::{
    middleware::{fullpage_render::FullPageRender, htmx::HtmxHeaders, login_guard::LoginGuard},
    models::user::{Role, User},
    users::UserListTemplate,
};

pub fn user_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(web::resource("/{id}").route(web::delete().to(delete)))
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
    name: web::Path<String>,
) -> impl Responder {
    let deleted = User::delete(&db, &name).await;
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

impl From<UserForm> for User {
    fn from(form: UserForm) -> Self {
        User {
            name: form.name,
            password: form.password,
            email: form.email,
            role_id: form.role_id.unwrap_or(Role::User as i32),
            source: crate::models::user::UserSource::Local,
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
    let user: User = form.into_inner().into();
    user.insert(&db).await;
    let users = User::list(&db).await;
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
    name: web::Path<String>,
) -> impl Responder {
    let user = User::get_by_name(&db, &name).await;
    if let Some(user) = user {
        let template = UserEditTemplate { user };
        HttpResponse::Ok().body(template.render().unwrap())
    } else {
        HttpResponse::NotFound().body("User not found")
    }
}
