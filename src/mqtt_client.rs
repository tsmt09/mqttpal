use crate::{
    middleware::{fullpage_render::FullPageRender, login_guard::LoginGuard},
    models::mqtt_client::{MqttClient, NewMqttClient},
    mqtt::MqttClientManager,
    mqtt_clients::MqttClientListTemplate,
    subscribe,
};
use actix_web::{web, HttpResponse, Responder};
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

pub fn client_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mqtt_client")
            .configure(subscribe::subscribe_scoped)
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get).wrap(FullPageRender))
                    .route(web::delete().to(delete)),
            )
            .service(web::resource("/").route(web::post().to(post)))
            .service(web::resource("/{id}/publish").route(web::post().to(post_publish))),
    );
}

async fn post(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    form: web::Form<MqttClientForm>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let new_client: NewMqttClient = form.into_inner().into();
    let client = new_client.insert(&mut conn);
    let _ = mqtt.register_client(client.id, client.url).await;
    let mqtt_clients = MqttClient::list(&mut conn);
    let template = MqttClientListTemplate { mqtt_clients };
    HttpResponse::Ok().body(template.render().unwrap())
}

async fn delete(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let client = MqttClient::get(&mut conn, *id);
    if client.is_some() {
        let _ = mqtt.unregister_client(*id).await;
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

#[derive(Template)]
#[template(path = "mqtt_client.html")]
struct MqttClientTemplate {
    id: i32,
    name: String,
    uri: String,
    connected: bool,
}

//#[get("/{id}")]
async fn get(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    id: web::Path<i32>,
) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let db_client = MqttClient::get(&mut conn, *id);
    if let Some(db_client) = db_client {
        let template = MqttClientTemplate {
            id: db_client.id,
            name: db_client.name,
            uri: db_client.url,
            connected: mqtt.connected(db_client.id).await,
        };
        HttpResponse::Ok().body(template.render().unwrap())
    } else {
        HttpResponse::Ok().body("Client not found")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MqttClientPublishForm {
    topic: String,
    payload: String,
}

async fn post_publish(
    _: LoginGuard,
    mqtt: web::Data<MqttClientManager>,
    form: web::Form<MqttClientPublishForm>,
    id: web::Path<i32>,
) -> impl Responder {
    let form = form.into_inner();
    let _ = mqtt
        .publish(*id, form.topic, Vec::from(form.payload.as_bytes()))
        .await;
    HttpResponse::Ok().body("okay")
}
