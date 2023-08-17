use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Role {
    pub id: i32,
    pub name: String,
}

impl Role {
    pub fn list(conn: &mut diesel::SqliteConnection) -> Vec<Role> {
        use crate::schema::roles::dsl::*;
        roles
            .select(Role::as_select())
            .load(conn)
            .expect("Error loading roles!")
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewRole {
    pub name: String,
}

impl NewRole {
    pub fn insert(&self, conn: &mut diesel::SqliteConnection) -> Role {
        use crate::schema::roles::dsl::*;
        let res = diesel::insert_into(roles)
            .values(self)
            .returning(Role::as_returning())
            .get_result(conn);
        match res {
            Ok(ok) => ok,
            Err(e) => {
                panic!("Error inserting role: {:?}", e);
            }
        }
    }
}
