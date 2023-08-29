use crate::{
    middleware::login_guard::LoginGuard,
    models::mqtt_client::{MqttClient, NewMqttClient},
    mqtt::MqttClientManager,
    mqtt_clients::MqttClientListTemplate,
};
use actix_web::{delete, post, web, HttpResponse, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct MqttClientForm {
    name: String,
    url: String,
}

impl From<MqttClientForm> for NewMqttClient {
    fn from(form: MqttClientForm) -> Self {
        NewMqttClient {
            name: form.name,
            url: form.url,
        }
    }
}

#[post("/mqtt_client/")]
async fn post(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    form: web::Form<MqttClientForm>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let new_client: NewMqttClient = form.into_inner().into();
    let _ = new_client.insert(&mut conn);
    let _ = mqtt.register_client(new_client.name, new_client.url).await;
    let mqtt_clients = MqttClient::list(&mut conn);
    let template = MqttClientListTemplate { mqtt_clients };
    HttpResponse::Ok().body(template.render().unwrap())
}

#[delete("/mqtt_client/{id}")]
async fn delete(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let client = MqttClient::get(&mut conn, *id);
    if let Some(client) = client {
        let _ = mqtt.unregister_client(client.name.clone()).await;
        let deleted = MqttClient::delete(&mut conn, *id);
        if deleted {
            HttpResponse::Ok().body("")
        } else {
            HttpResponse::NotFound().body("MqttClient not found")
        }
    } else {
        HttpResponse::NotFound().body("MqttClient not found")
    }
}
