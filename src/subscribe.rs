use crate::middleware::fullpage_render::FullPageRender;
use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self};
use askama::Template;
use serde::{Deserialize, Serialize};

/// Define HTTP actor
struct WsSubscription {
    topic: String,
    tx: tokio::sync::broadcast::Sender<rumqttc::Event>,
}

pub fn subscribe_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("{name}/subscribe")
            .service(web::resource("/ws").route(web::get().to(ws)))
            .service(web::resource("").route(web::get().to(get).wrap(FullPageRender))),
    );
}

impl Actor for WsSubscription {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSubscription {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text({
                format!(
                    "<div id=\"responseBox\" hx-swap-oob=\"beforeend\">{}</div>",
                    text
                )
            }),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(_)) => {
                log::info!("Closing websocket connection");
                ctx.close(None)
            }
            _ => (),
        }
    }
}

async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    name: web::Path<String>,
    query: web::Query<SubscriptionForm>,
    mqtt_clients: web::Data<crate::mqtt::MqttClientManager>,
) -> Result<HttpResponse, Error> {
    if let Some(tx) = mqtt_clients.tx(&name).await {
        let topic = query.topic.clone();
        ws::start(WsSubscription { topic, tx }, &req, stream)
    } else {
        log::info!("Client not found");
        return Ok(HttpResponse::NotFound().body("Client not found"));
    }
}

#[derive(Template)]
#[template(path = "subscribe.html")]
struct SubscriptionTemplate {
    name: String,
    topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubscriptionForm {
    topic: String,
}

async fn get(name: web::Path<String>, query: web::Query<SubscriptionForm>) -> HttpResponse {
    HttpResponse::Ok().body(
        SubscriptionTemplate {
            name: name.clone(),
            topic: query.topic.clone(),
        }
        .render()
        .unwrap(),
    )
}
