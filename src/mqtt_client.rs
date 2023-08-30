use crate::{
    middleware::{fullpage_render::FullPageRender, login_guard::LoginGuard},
    models::mqtt_client::{MqttClient, NewMqttClient},
    mqtt::MqttClientManager,
    mqtt_clients::MqttClientListTemplate,
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
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get).wrap(FullPageRender))
                    .route(web::delete().to(delete)),
            )
            .service(web::resource("/").route(web::post().to(post))),
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
    let _ = new_client.insert(&mut conn);
    let _ = mqtt.register_client(new_client.name, new_client.url).await;
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

#[derive(Template)]
#[template(path = "mqtt_client.html")]
struct MqttClientTemplate {
    id: i32,
}

//#[get("/{id}")]
async fn get(_: LoginGuard, id: web::Path<i32>) -> impl Responder {
    let template = MqttClientTemplate { id: *id };
    HttpResponse::Ok().body(template.render().unwrap())
}
