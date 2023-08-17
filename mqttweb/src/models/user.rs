use diesel::prelude::*;
use crate::models::role::Role;

#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
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
    pub fn list_with_roles(conn: &mut diesel::SqliteConnection) -> Vec<(User, Vec<Role>)> {
        use crate::schema::roles::dsl::*;
        use crate::schema::user_role::dsl::*;
        use crate::schema::users::dsl::*;
        let res = users
            .select(User::as_select())
            .load::<User>(conn)
            .expect("Error loading users!");
        let mut users_with_roles = Vec::new();
        for user in res {
            let res_roles = user_role
                .inner_join(roles)
                .filter(user_id.eq(user.id))
                .select(Role::as_select())
                .load(conn)
                .expect("Error loading roles!");
            users_with_roles.push((user, res_roles));
        }
        users_with_roles
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
    pub fn add_role(conn: &mut diesel::SqliteConnection, uid: i32, rid: i32) -> bool {
        use crate::schema::user_role::dsl::*;
        log::info!("Adding role {} to user {}", rid, uid);
        let res = diesel::insert_into(user_role)
            .values((user_id.eq(uid), role_id.eq(rid)))
            .execute(conn);
        match res {
            Ok(ok) => ok > 0,
            Err(e) => {
                log::error!("Error adding role to user: {:?}", e);
                false
            }
        }
    }
    fn remove_role(conn: &mut diesel::SqliteConnection, uid: i32, rid: i32) -> bool {
        use crate::schema::user_role::dsl::*;
        log::info!("Removing role {} from user {}", rid, uid);
        let res = diesel::delete(user_role)
            .filter(user_id.eq(uid).and(role_id.eq(rid)))
            .execute(conn);
        match res {
            Ok(ok) => ok > 0,
            Err(e) => {
                log::error!("Error removing role from user: {:?}", e);
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
