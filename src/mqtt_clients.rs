use actix_web::{get, web, HttpResponse, Responder};
use askama::Template;

use crate::{middleware::login_guard::LoginGuard, models::mqtt_client::MqttClient};

#[derive(Template)]
#[template(path = "mqtt_clients.html")]
pub struct MqttClientListTemplate {
    pub mqtt_clients: Vec<MqttClient>,
}

#[get("/")]
async fn get(_: LoginGuard, db: web::Data<crate::DbPool>) -> impl Responder {
    let mut conn = db.get().expect("no connection available");
    let mqtt_clients = MqttClient::list(&mut conn);
    let template = MqttClientListTemplate { mqtt_clients };
    HttpResponse::Ok().body(template.render().unwrap())
}