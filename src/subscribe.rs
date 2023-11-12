use crate::{
    middleware::{fullpage_render::FullPageRender, login_guard::LoginGuard},
    models::mqtt_client::MqttClient,
    mqtt::{MqttClientActor, MqttClientManager, MqttMessage},
};
use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler, System};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws::{self, CloseReason};
use askama::Template;
use serde::{Deserialize, Serialize};

/// Define HTTP actor
struct WsSubscription {
    client_name: String,
    ws_id: i32,
    addr: Addr<MqttClientActor>,
}

pub fn subscribe_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("{name}/subscribe")
            .service(web::resource("/ws").route(web::get().to(ws)))
            .service(
                web::resource("")
                    .route(web::delete().to(post_unsubscribe))
                    .route(web::post().to(post_subscribe)),
            ),
    );
}

impl Actor for WsSubscription {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("started ws -> mqtt actor");
        let addr = ctx.address().recipient();
        self.addr.do_send(MqttMessage::Sub((self.ws_id, addr)));
    }
}

#[derive(Template)]
#[template(path = "mqtt_message.html")]
struct MessageTemplate {
    topic: String,
    payload: String,
}

impl Handler<MqttMessage> for WsSubscription {
    type Result = ();
    fn handle(&mut self, msg: MqttMessage, ctx: &mut Self::Context) -> Self::Result {
        log::info!("Got mqtt message: {:?}", msg);
        match msg {
            MqttMessage::Message(publsh) => {
                let topic = publsh.topic;
                let payload = String::from_utf8(publsh.payload.into()).unwrap();
                let response = MessageTemplate { topic, payload }.render().unwrap();
                ctx.text(response);
            }
            MqttMessage::Disconnect => {
                log::info!("Disconnect from mqtt manager!");
                ctx.close(None);
            }
            _ => (),
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSubscription {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => {
                log::info!("Closing websocket connection");
                self.addr.do_send(MqttMessage::Unsub(self.ws_id));
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
    mqtt_clients: web::Data<crate::mqtt::MqttClientManager>,
) -> Result<HttpResponse, Error> {
    let ws_id = rand::random::<i32>();
    if let Some(addr) = mqtt_clients.get_client_actor_addr(&name).await {
        ws::start(
            WsSubscription {
                client_name: name.clone(),
                ws_id,
                addr,
            },
            &req,
            stream,
        )
    } else {
        Ok(HttpResponse::NotFound().body("MqttClient not found"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MqttClientSubQuery {
    topic: String,
}

#[derive(Template)]
#[template(path = "mqtt_client_subs.html")]
struct MqttClientSubTemplate {
    name: String,
    topics: Vec<String>,
}

async fn post_subscribe(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    form: web::Form<MqttClientSubQuery>,
    name: web::Path<String>,
) -> impl Responder {
    let topic = form.into_inner().topic;
    let _ = mqtt.subscribe(&name, &topic).await;
    MqttClient::subscribe(&db, &name, &topic).await;
    let topics = MqttClient::topics(&db, &name).await;
    let template = MqttClientSubTemplate {
        name: name.into_inner(),
        topics,
    };
    HttpResponse::Ok().body(template.render().unwrap())
}

async fn post_unsubscribe(
    _: LoginGuard,
    db: web::Data<crate::DbPool>,
    mqtt: web::Data<MqttClientManager>,
    query: web::Query<MqttClientSubQuery>,
    name: web::Path<String>,
) -> impl Responder {
    let topic = query.into_inner().topic;
    let _ = mqtt.unsubscribe(&name, &topic).await;
    MqttClient::unsubscribe(&db, &name, &topic).await;
    let topics = MqttClient::topics(&db, &name).await;
    let template = MqttClientSubTemplate {
        name: name.into_inner(),
        topics,
    };
    HttpResponse::Ok().body(template.render().unwrap())
}
