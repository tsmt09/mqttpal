use clap::Parser;
use mqttmonitor::connect;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1")]
    server: String,
    #[arg(short, long, default_value_t = 1883)]
    port: u16,
    #[arg(short, long, default_value_t = 5)]
    keep_alive_secs: u64,
    #[arg(short, long, default_value_t = 10)]
    channel_capacity: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = Cli::parse();
    let (mut _client, mut _connection) = connect(
        cli_args.server.as_str(),
        cli_args.port,
        cli_args.keep_alive_secs,
        cli_args.channel_capacity,
    )
    .unwrap();
    Ok(())
}
