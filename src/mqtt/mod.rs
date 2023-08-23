use std::{collections::HashMap, sync::Arc};

use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use tokio::sync::Mutex;

#[derive(Clone)]
struct MqttClient {
    client: AsyncClient,
    tx: tokio::sync::broadcast::Sender<Event>,
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
    pub async fn register_client(
        &mut self,
        client_id: String,
        mqtt_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Registering client: {} with url: {}", client_id, mqtt_url);
        let options = MqttOptions::parse_url(&mqtt_url)?;
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        let (tx, _) = tokio::sync::broadcast::channel::<Event>(10);

        let tx2 = tx.clone();
        let cid = client_id.clone();

        tokio::spawn(async move {
            loop {
                let event = eventloop.poll().await.unwrap();
                log::debug!("Event in client {}: {:?}", cid, event);
                if let Event::Outgoing(rumqttc::Outgoing::Disconnect) = event {
                    let _ = tx2.send(event);
                    log::info!("Client {} disconnected!", cid);
                    break;
                }
                let _ = tx2.send(event);
            }
        });

        let client = MqttClient { client, tx };
        self.clients.lock().await.insert(client_id, client);
        Ok(())
    }
    #[allow(dead_code)]
    pub async fn subscribe(
        &mut self,
        client_id: String,
        topic: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Subscribing client: {} to topic: {}", client_id, topic);
        let mut clients = self.clients.lock().await;
        let client = clients.get_mut(&client_id).unwrap();
        client.client.subscribe(topic, QoS::AtLeastOnce).await?;
        Ok(())
    }
    #[allow(dead_code)]
    pub async fn tx(&self, client_id: String) -> Option<tokio::sync::broadcast::Sender<Event>> {
        let clients = self.clients.lock().await;
        let client = clients.get(&client_id)?;
        Some(client.tx.clone())
    }
}
