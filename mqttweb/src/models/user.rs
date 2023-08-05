use diesel::prelude::*;
use serde::Deserialize;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub password: String,
}

impl User {
    pub fn check(
        conn: &mut diesel::SqliteConnection,
        check_name: &str,
        check_password: &str,
    ) -> bool {
        use crate::schema::users::dsl::*;
        let res = users
            .filter(name.eq(check_name).and(password.eq(check_password)))
            .select(User::as_select())
            .load(conn);
        match res {
            Ok(ok) => !ok.is_empty(),
            Err(e) => {
                log::error!("Error querying user: {:?}", e);
                false
            }
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewUser<'n> {
    pub name: &'n str,
    pub email: Option<&'n str>,
    pub password: &'n str,
}

impl<'n> NewUser<'n> {
    pub fn insert(&self, conn: &mut SqliteConnection) -> User {
        diesel::insert_into(crate::schema::users::table)
            .values(self)
            .returning(User::as_returning())
            .get_result(conn)
            .expect("Cannot insert user!")
    }
}

pub struct Role {
    name: String,
    permissions: Vec<String>,
}
