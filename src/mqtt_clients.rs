use actix_web::{get, web, HttpResponse, Responder};
use askama::Template;

use crate::{
    middleware::{fullpage_render::FullPageRender, login_guard::LoginGuard},
    models::mqtt_client::MqttClient,
};

pub fn clients_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mqtt_clients")
            .wrap(FullPageRender)
            .service(get),
    );
}

#[derive(Template)]
#[template(path = "mqtt_clients.html")]
pub struct MqttClientListTemplate {
    pub mqtt_clients: Vec<MqttClient>,
}

#[get("/")]
async fn get(_: LoginGuard, db: web::Data<crate::DbPool>) -> impl Responder {
    let mqtt_clients = MqttClient::list(&db).await;
    let template = MqttClientListTemplate { mqtt_clients };
    HttpResponse::Ok().body(template.render().unwrap())
}
