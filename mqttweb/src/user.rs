use actix_web::{delete, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};

use crate::{
    middleware::{htmx::HtmxHeaders, login_guard::LoginGuard},
    models::user::{NewUser, Role, User},
};

#[delete("/user/{id}")]
async fn delete_user(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let deleted = User::delete(&mut conn, *id);
    if deleted {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::NotFound().body("User not found")
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

#[post("/user/")]
async fn post(
    _: LoginGuard,
    req: HttpRequest,
    db: web::Data<crate::DbPool>,
    form: web::Form<UserForm>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let new_user: NewUser = form.into_inner().into();
    let user = new_user.insert(&mut conn);
    if let Some(htmx) = req.extensions_mut().get_mut::<HtmxHeaders>() {
        if htmx.request() {
            let template = UserRowTemplate { user }.render().unwrap();
            return HttpResponse::Ok().body(template);
        }
    }
    HttpResponse::Ok().finish()
}
