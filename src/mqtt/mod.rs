use std::{collections::HashMap, sync::Arc};

use actix::{Actor, ActorContext, Addr, Context, Handler, Message, Recipient};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, Publish, QoS};
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Debug, Clone)]
pub enum MqttMessage {
    Message(Publish),
    Sub((i32, Recipient<MqttMessage>)),
    Unsub(i32),
    Disconnect,
}

impl Message for MqttMessage {
    type Result = ();
}

pub struct MqttClientActor {
    ws_subs: HashMap<i32, Recipient<MqttMessage>>,
}

impl MqttClientActor {
    fn reg_ws_sub(&mut self, ws_id: i32, addr: Recipient<MqttMessage>) {
        self.ws_subs.insert(ws_id, addr);
    }
    fn reg_ws_unsub(&mut self, ws_id: i32) {
        self.ws_subs.remove(&ws_id);
    }
}

impl Actor for MqttClientActor {
    type Context = Context<Self>;
}

impl Handler<MqttMessage> for MqttClientActor {
    type Result = ();
    fn handle(&mut self, msg: MqttMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            MqttMessage::Message(_) => {
                log::info!(
                    "distributing message within {} ws clients",
                    self.ws_subs.len()
                );
                for addr in self.ws_subs.values() {
                    addr.do_send(msg.clone());
                }
            }
            MqttMessage::Sub((ws_id, addr)) => {
                log::info!("Registering ws_id: {} for mqtt messages", ws_id);
                self.reg_ws_sub(ws_id, addr);
            }
            MqttMessage::Unsub(ws_id) => {
                log::info!("Unregistering ws_id: {} for mqtt messages", ws_id);
                self.reg_ws_unsub(ws_id);
            }
            MqttMessage::Disconnect => {
                log::info!("Stopping Actor!");
                ctx.stop()
            }
        }
    }
}

struct MqttClient {
    client: AsyncClient,
    handle: JoinHandle<()>,
    addr: Addr<MqttClientActor>,
}

impl Drop for MqttClient {
    fn drop(&mut self) {
        log::info!("Dropping client!");
        self.client.try_disconnect().unwrap();
    }
}

#[derive(Clone)]
pub struct MqttClientManager {
    clients: Arc<Mutex<HashMap<String, MqttClient>>>,
}

impl MqttClientManager {
    pub fn new() -> Self {
        MqttClientManager {
            clients: Arc::new(Mutex::new(HashMap::<String, MqttClient>::new())),
        }
    }

    pub async fn get_client_actor_addr(
        &self,
        client_name: &String,
    ) -> Option<Addr<MqttClientActor>> {
        let clients = self.clients.lock().await;
        let client = clients.get(client_name)?;
        Some(client.addr.clone())
    }

    pub async fn register_client(
        &self,
        client_name: String,
        mqtt_url: String,
        topics: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("registering client {} with url {}", client_name, mqtt_url);
        let mqtt_url = if !mqtt_url.contains("?client_id") {
            format!("{}?client_id={}", mqtt_url, client_name)
        } else {
            mqtt_url
        };
        let mut options = MqttOptions::parse_url(&mqtt_url)?;
        options.set_max_packet_size(100000, 100000);
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        for topic in topics {
            client.subscribe(&topic, QoS::AtLeastOnce).await?;
        }
        let cid = client_name.clone();
        let addr_handle = MqttClientActor {
            ws_subs: HashMap::new(),
        }
        .start();
        let addr = addr_handle.clone();
        let handle = tokio::spawn(async move {
            log::info!("Client {} connected!", cid);
            loop {
                let event = eventloop.poll().await;
                match event {
                    Ok(Event::Incoming(inc)) => match inc {
                        Packet::Publish(publish) => {
                            log::info!("Client {} got message: {:?}", cid, publish);
                            let _ = addr_handle.send(MqttMessage::Message(publish)).await;
                        }
                        Packet::ConnAck(_) => {
                            log::info!("Client {} got ConnAck", cid);
                        }
                        Packet::Disconnect => {
                            log::info!("Server sent Disconnect");
                            let _ = addr_handle.send(MqttMessage::Disconnect).await;
                            break;
                        }
                        _ => {}
                    },
                    Ok(Event::Outgoing(out)) => {
                        if out == rumqttc::Outgoing::Disconnect {
                            log::info!("Client {} sent Disconnect", cid);
                        }
                    }
                    Err(e) => {
                        log::info!("Client {} got error: {:?}", cid, e);
                        break;
                    }
                }
            }
        });

        let mqtt_client = MqttClient {
            client,
            handle,
            addr,
        };
        self.clients.lock().await.insert(client_name, mqtt_client);
        Ok(())
    }
    pub async fn unregister_client(&self, client_name: &String) {
        log::info!("Unregistering client: {}", &client_name);
        let mut clients = self.clients.lock().await;
        let client = clients.remove(client_name);
        if let Some(client) = client {
            client.client.disconnect().await.unwrap();
        }
    }
    #[allow(dead_code)]
    pub async fn subscribe(
        &self,
        client_name: &String,
        topic: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Subscribing client: {} to topic: {}", client_name, topic);
        let mut clients = self.clients.lock().await;
        let client = clients.get_mut(client_name).unwrap();
        client.client.subscribe(topic, QoS::AtLeastOnce).await?;
        Ok(())
    }
    #[allow(dead_code)]
    pub async fn unsubscribe(
        &self,
        client_name: &String,
        topic: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Unsubscribing client: {} to topic: {}", client_name, topic);
        let mut clients = self.clients.lock().await;
        let client = clients.get_mut(client_name).unwrap();
        client.client.unsubscribe(topic).await?;
        Ok(())
    }
    pub async fn connected(&self, client_name: &String) -> bool {
        let clients = self.clients.lock().await;
        let client = clients.get(client_name).unwrap();
        !client.handle.is_finished()
    }
    pub async fn publish(
        &self,
        client_name: &String,
        topic: String,
        payload: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Publishing to client: {} to topic: {}", client_name, topic);
        let mut clients = self.clients.lock().await;
        let client = clients.get_mut(client_name).unwrap();
        client
            .client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await?;
        Ok(())
    }
}
