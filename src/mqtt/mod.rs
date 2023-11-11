use std::{collections::HashMap, sync::Arc};

use rumqttc::{AsyncClient, Event, MqttOptions, QoS, tokio_rustls::client};
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
        &self,
        client_name: String,
        mqtt_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Registering client: {} with url: {}", client_name, mqtt_url);
        let options = MqttOptions::parse_url(&mqtt_url)?;
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        let (tx, _) = tokio::sync::broadcast::channel::<Event>(10);

        let tx2 = tx.clone();
        let cid = client_name.clone();

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
        self.clients.lock().await.insert(client_name, client);
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
        &mut self,
        client_name: String,
        topic: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Subscribing client: {} to topic: {}", client_name, topic);
        let mut clients = self.clients.lock().await;
        let client = clients.get_mut(&client_name).unwrap();
        client.client.subscribe(topic, QoS::AtLeastOnce).await?;
        Ok(())
    }
    #[allow(dead_code)]
    pub async fn tx(&self, client_name: &String) -> Option<tokio::sync::broadcast::Sender<Event>> {
        let clients = self.clients.lock().await;
        log::debug!("{:?}", clients.keys().collect::<Vec<&String>>());
        let client = clients.get(client_name)?;
        Some(client.tx.clone())
    }
    pub async fn connected(&self, _client_name: &String) -> bool {
        // TODO: MqttClient has no connected method
        true
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
