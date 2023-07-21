use chrono::{DateTime, Utc, NaiveDateTime};
use serde::Deserialize;
use diesel::prelude::*;

#[derive(Clone, Debug, Deserialize)]
pub struct UserForm {
    email: String,
    password: String,
    remember: Option<String>,
}

#[derive(Queryable,Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub remember: bool,
    pub role_id: i32,
    pub created_at: NaiveDateTime, 
    pub updated_at: NaiveDateTime,
}

pub struct Role {
    name: String,
    permissions: Vec<String>
}