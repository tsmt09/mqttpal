use rumqttc::{Client, Connection, MqttOptions};
use std::time::Duration;

//should be refactored into MqttOptions like struct
pub fn connect(
    server: &str,
    port: u16,
    keep_alive_secs: u64,
    channel_capacity: usize,
) -> Result<(Client, Connection), Box<dyn std::error::Error>> {
    let mut options = MqttOptions::new("testing mqtt", server, port);
    options.set_keep_alive(Duration::from_secs(keep_alive_secs));
    Ok(Client::new(options, channel_capacity))
}
