use crate::{
    middleware::{fullpage_render::FullPageRender, login_guard::LoginGuard},
    models::mqtt_client::MqttClient,
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

impl From<MqttClientForm> for MqttClient {
    fn from(form: MqttClientForm) -> Self {
        MqttClient {
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
    let client: MqttClient = form.into_inner().into();
    client.insert(&db).await;
    let _ = mqtt.register_client(client.name, client.url).await;
    let mqtt_clients = MqttClient::list(&db).await;
    let template = MqttClientListTemplate { mqtt_clients };
    HttpResponse::Ok().body(template.render().unwrap())
}

async fn delete(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    name: web::Path<String>,
) -> impl Responder {
    let deleted = MqttClient::delete(&db, &name).await;
    if deleted {
        let _ = mqtt.unregister_client(&name).await;
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::NotFound().body("MqttClient not found")
    }
}

#[derive(Template)]
#[template(path = "mqtt_client.html")]
struct MqttClientTemplate {
    name: String,
    uri: String,
    connected: bool,
}

//#[get("/{id}")]
async fn get(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    name: web::Path<String>,
) -> impl Responder {
    let db_client = MqttClient::get_by_name(&db, &name).await;
    if let Some(db_client) = db_client {
        let template = MqttClientTemplate {
            name: db_client.name.clone(),
            uri: db_client.url.clone(),
            connected: mqtt.connected(&db_client.name).await,
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
    name: web::Path<String>,
) -> impl Responder {
    let form = form.into_inner();
    let _ = mqtt
        .publish(&name, form.topic, Vec::from(form.payload.as_bytes()))
        .await;
    HttpResponse::Ok().body("okay")
}
