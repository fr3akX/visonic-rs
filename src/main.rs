use std::error::Error;

use clap::Parser;
use log::{error, info, LevelFilter};
use serde::Deserialize;

use crate::mqtt::mqtt_handler::MqttHandlerConfig;
use crate::visonic::visonic::{Visonic, VisonicErr};

mod mqtt;
mod visonic;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(short, long)]
    config: String,
}

#[derive(Deserialize)]
struct Configuration {
    mqtt: MqttHandlerConfig,
    visonic: Visonic,
}

fn read_config(config_path: &String) -> std::io::Result<Configuration> {
    let s = std::fs::read_to_string(config_path)?;
    let c: Configuration = toml::from_str(s.as_str()).unwrap();
    Ok(c)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::formatted_builder().filter_level(LevelFilter::Debug).init();

    let args = CliArgs::parse();
    let config = read_config(&args.config).unwrap();

    info!("Loading config from");

    let visonic = config.visonic.login().await.unwrap();

    let s  = visonic.status().await.unwrap();
    info!("STATUS: {}", s.connected);

    let s  = visonic.status_txt().await.unwrap();
    info!("STATUS: {}", s);

    let s  = visonic.alarms().await.unwrap();
    info!("alarms: {:?}", s);

    let s  = visonic.alerts().await.unwrap();
    info!("alerts: {:?}", s);

    let s  = visonic.troubles().await.unwrap();
    info!("troubles: {:?}", s);

    let s  = visonic.panel_info().await.unwrap();
    info!("panel_info: {}", s);

    let s  = visonic.wakeup_sms().await.unwrap();
    info!("wakeup_sms: {:?}", s);

    let s  = visonic.devices().await.unwrap();
    info!("devices: {:?}", s);

    let s  = visonic.locations().await.unwrap();
    info!("locations: {:?}", s);

    let s  = visonic.events().await.unwrap();
    info!("events: {:?}", s);

    let mut connection = config.mqtt.connect().await.expect("Could not connect to MQTT");
    info!("Connected to MQTT broker");

    connection.handle(|msg| async {
        fn handle(cmd: String, r: Result<(), VisonicErr>) -> Option<String> {
            return match r {
                Ok(_) => Some(cmd),
                Err(err) => {
                    error!("Failure {}: {}", cmd, err);
                    Some("ERROR".to_string())
                }
            }
        }

        match msg.payload {
            s if s.eq("AWAY") => {
                let arm = visonic.arm().await;
                return handle("AWAY".to_string(), arm);
            },
            s if s.eq("DISARM") => {
                let arm = visonic.disarm().await;
                return handle("DISARM".to_string(), arm);
            },
            s if s.eq("NIGHT") => {
                let arm = visonic.arm_night().await;
                return handle("NIGHT".to_string(), arm);
            },
            s if s.eq("STAY") => {
                let arm = visonic.arm_stay().await;
                return handle("STAY".to_string(), arm);
            },
            s => {
                info!("unknown mqtt command: {}", s.to_string());
                None
            },
        }
    }).await;

    Ok(())
}
