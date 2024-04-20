use bb8_redis::redis::cmd;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MqttClient {
    pub name: String,
    pub url: String,
}

impl MqttClient {
    pub async fn list(pool: &crate::DbPool) -> Vec<MqttClient> {
        let mut conn = pool.get().await.expect("no connection available");
        let mut clients_hash: Vec<String> = cmd("HGETALL")
            .arg("mqtt_clients")
            .query_async(&mut *conn)
            .await
            .expect("Cannot query mqtt_clients from redis");
        let mut clients: Vec<MqttClient> = Vec::new();
        while let Some(client) = clients_hash.pop() {
            let client: MqttClient =
                serde_json::from_str(&client).expect("Cannot deserialize mqtt_client");
            clients.push(client);
            let _ = clients_hash.pop();
        }
        clients
    }
    pub async fn insert(&self, pool: &crate::DbPool) {
        let mut conn = pool.get().await.expect("no connection available");
        let client_json = serde_json::to_string(&self).expect("Cannot serialize mqtt_client");
        let _: i32 = cmd("HSET")
            .arg("mqtt_clients")
            .arg(&self.name)
            .arg(client_json)
            .query_async(&mut *conn)
            .await
            .expect("Cannot insert mqtt_client");
    }
    pub async fn get_by_name(pool: &crate::DbPool, name: &str) -> Option<MqttClient> {
        let mut conn = pool.get().await.expect("no connection available");
        let client: Option<String> = cmd("HGET")
            .arg("mqtt_clients")
            .arg(name)
            .query_async(&mut *conn)
            .await
            .expect("Cannot query mqtt_clients from redis");
        if let Some(client) = client {
            let client: MqttClient =
                serde_json::from_str(&client).expect("Cannot deserialize mqtt_client");
            Some(client)
        } else {
            None
        }
    }
    pub async fn delete(pool: &crate::DbPool, name: &str) -> bool {
        let mut conn = pool.get().await.expect("no connection available");
        let deleted: i32 = cmd("HDEL")
            .arg("mqtt_clients")
            .arg(name)
            .query_async(&mut *conn)
            .await
            .expect("Cannot delete mqtt_client");
        let _: i32 = cmd("DEL")
            .arg(format!("mqtt_client:{}:topics", name))
            .query_async(&mut *conn)
            .await
            .expect("Cannot delete mqtt_client topics");
        deleted > 0
    }
    pub async fn topics(pool: &crate::DbPool, name: &str) -> Vec<String> {
        let mut conn = pool.get().await.expect("no connection available");
        let topics: Vec<String> = cmd("SMEMBERS")
            .arg(format!("mqtt_client:{}:topics", name))
            .query_async(&mut *conn)
            .await
            .expect("Cannot query mqtt_client topics from redis");
        topics
    }
    pub async fn subscribe(pool: &crate::DbPool, name: &str, topic: &str) -> bool {
        let mut conn = pool.get().await.expect("no connection available");
        let subscribed: i32 = cmd("SADD")
            .arg(format!("mqtt_client:{}:topics", name))
            .arg(topic)
            .query_async(&mut *conn)
            .await
            .expect("Cannot subscribe mqtt_client");
        subscribed > 0
    }
    pub async fn unsubscribe(pool: &crate::DbPool, name: &str, topic: &str) -> bool {
        let mut conn = pool.get().await.expect("no connection available");
        let unsubscribed: i32 = cmd("SREM")
            .arg(format!("mqtt_client:{}:topics", name))
            .arg(topic)
            .query_async(&mut *conn)
            .await
            .expect("Cannot unsubscribe mqtt_client");
        unsubscribed > 0
    }
}
