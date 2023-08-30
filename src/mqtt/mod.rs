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
    clients: Arc<Mutex<HashMap<i32, MqttClient>>>,
}

impl MqttClientManager {
    pub fn new() -> Self {
        MqttClientManager {
            clients: Arc::new(Mutex::new(HashMap::<i32, MqttClient>::new())),
        }
    }
    pub async fn register_client(
        &self,
        client_id: i32,
        mqtt_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Registering client: {} with url: {}", client_id, mqtt_url);
        let options = MqttOptions::parse_url(&mqtt_url)?;
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        let (tx, _) = tokio::sync::broadcast::channel::<Event>(10);

        let tx2 = tx.clone();
        let cid = client_id;

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
    pub async fn unregister_client(&self, client_id: i32) {
        log::info!("Unregistering client: {}", client_id);
        let mut clients = self.clients.lock().await;
        let client = clients.remove(&client_id);
        if let Some(client) = client {
            client.client.disconnect().await.unwrap();
        }
    }
    #[allow(dead_code)]
    pub async fn subscribe(
        &mut self,
        client_id: i32,
        topic: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Subscribing client: {} to topic: {}", client_id, topic);
        let mut clients = self.clients.lock().await;
        let client = clients.get_mut(&client_id).unwrap();
        client.client.subscribe(topic, QoS::AtLeastOnce).await?;
        Ok(())
    }
    #[allow(dead_code)]
    pub async fn tx(&self, client_id: i32) -> Option<tokio::sync::broadcast::Sender<Event>> {
        let clients = self.clients.lock().await;
        let client = clients.get(&client_id)?;
        Some(client.tx.clone())
    }
    pub async fn connected(&self, _client_id: i32) -> bool {
        // TODO: MqttClient has no connected method
        true
    }
    pub async fn publish(
        &self,
        client_id: i32,
        topic: String,
        payload: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Publishing to client: {} to topic: {}", client_id, topic);
        let mut clients = self.clients.lock().await;
        let client = clients.get_mut(&client_id).unwrap();
        client
            .client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await?;
        Ok(())
    }
}
