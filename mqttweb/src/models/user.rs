use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
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
    pub fn list(conn: &mut diesel::SqliteConnection) -> Vec<User> {
        use crate::schema::users::dsl::*;
        users
            .select(User::as_select())
            .load(conn)
            .expect("Error loading users!")
    }
    pub fn delete(conn: &mut diesel::SqliteConnection, user_id: i32) -> bool {
        use crate::schema::users::dsl::*;
        log::info!("Deleting user with id: {}", user_id);
        let res = diesel::delete(users.filter(id.eq(user_id))).execute(conn);
        match res {
            Ok(ok) => ok > 0,
            Err(e) => {
                log::error!("Error deleting user: {:?}", e);
                false
            }
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewUser {
    pub name: String,
    pub email: Option<String>,
    pub password: String,
}

impl NewUser {
    pub fn insert(&self, conn: &mut SqliteConnection) -> User {
        log::info!("Inserting user: {:?}", self);
        diesel::insert_into(crate::schema::users::table)
            .values(self)
            .returning(User::as_returning())
            .get_result(conn)
            .expect("Cannot insert user!")
    }
}

pub struct _Role {
    name: String,
    permissions: Vec<String>,
}
