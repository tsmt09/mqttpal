use crate::middleware::fullpage_render::FullPageRender;
use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self};
use askama::Template;
use serde::{Deserialize, Serialize};

/// Define HTTP actor
struct WsSubscription;

pub fn subscribe_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("{id}/subscribe")
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
            Ok(ws::Message::Text(text)) => ctx.text(format!(
                "<div id=\"responseBox\" hx-swap-oob=\"beforeend\">{}</div>",
                text
            )),
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
    _id: web::Path<i32>,
    _query: web::Query<SubscriptionForm>,
) -> Result<HttpResponse, Error> {
    ws::start(WsSubscription {}, &req, stream)
}

#[derive(Template)]
#[template(path = "subscribe.html")]
struct SubscriptionTemplate {
    id: i32,
    topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubscriptionForm {
    topic: String,
}

async fn get(id: web::Path<i32>, query: web::Query<SubscriptionForm>) -> HttpResponse {
    log::debug!("Test");
    HttpResponse::Ok().body(
        SubscriptionTemplate {
            id: *id,
            topic: query.topic.clone(),
        }
        .render()
        .unwrap(),
    )
}
