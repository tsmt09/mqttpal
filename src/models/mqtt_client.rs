use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::mqtt_clients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MqttClient {
    pub id: i32,
    pub name: String,
    pub url: String,
}

impl MqttClient {
    pub fn list(conn: &mut diesel::SqliteConnection) -> Vec<MqttClient> {
        use crate::schema::mqtt_clients::dsl::*;
        mqtt_clients
            .select(MqttClient::as_select())
            .load(conn)
            .expect("Error loading users!")
    }
    pub fn delete(conn: &mut diesel::SqliteConnection, mqtt_client_id: i32) -> bool {
        use crate::schema::mqtt_clients::dsl::*;
        log::info!("Deleting user with id: {}", mqtt_client_id);
        let res = diesel::delete(mqtt_clients.filter(id.eq(mqtt_client_id))).execute(conn);
        match res {
            Ok(ok) => ok > 0,
            Err(e) => {
                log::error!("Error deleting user: {:?}", e);
                false
            }
        }
    }
    pub fn get(conn: &mut diesel::SqliteConnection, mqtt_client_id: i32) -> Option<MqttClient> {
        use crate::schema::mqtt_clients::dsl::*;
        let res = mqtt_clients
            .filter(id.eq(mqtt_client_id))
            .select(MqttClient::as_select())
            .load(conn);
        match res {
            Ok(mut ok) => ok.pop(),
            Err(e) => {
                log::error!("Error querying user: {:?}", e);
                None
            }
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::mqtt_clients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMqttClient {
    pub name: String,
    pub url: String,
}

impl NewMqttClient {
    pub fn insert(&self, conn: &mut SqliteConnection) -> MqttClient {
        log::info!("Inserting mqtt_client: {:?}", self);
        diesel::insert_into(crate::schema::mqtt_clients::table)
            .values(self)
            .returning(MqttClient::as_returning())
            .get_result(conn)
            .expect("Cannot insert user!")
    }
}
