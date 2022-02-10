use std::error::Error;

use clap::Parser;
use log::{error, info, LevelFilter};
use serde::Deserialize;

use crate::mqtt::mqtt_handler::MqttHandlerConfig;
use crate::visonic::visonic::{AuthedVisonic, Visonic, VisonicErr};

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
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let args = CliArgs::parse();
    let config = read_config(&args.config).unwrap();

    info!("Loading config from");

    let visonic = config.visonic.login().await.unwrap();

    let s = visonic.status().await.unwrap();
    info!("STATUS: {}", s.connected);

    let s = visonic.status_txt().await.unwrap();
    info!("STATUS: {}", s);

    let s = visonic.alarms().await.unwrap();
    info!("alarms: {:?}", s);

    let s = visonic.alerts().await.unwrap();
    info!("alerts: {:?}", s);

    let s = visonic.troubles().await.unwrap();
    info!("troubles: {:?}", s);

    let panel_info = visonic.panel_info().await.unwrap();
    info!("panel_info: {}", panel_info);

    let s = visonic.wakeup_sms().await.unwrap();
    info!("wakeup_sms: {:?}", s);

    let s = visonic.devices().await.unwrap();
    info!("devices: {:?}", s);

    let s = visonic.locations().await.unwrap();
    info!("locations: {:?}", s);

    let s = visonic.events().await.unwrap();
    info!("events: {:?}", s);

    let mut connection = config
        .mqtt
        .connect()
        .await
        .expect("Could not connect to MQTT");
    info!("Connected to MQTT broker");

    connection
        .publish(config.mqtt.info_topic, panel_info)
        .await
        .unwrap();
    connection
        .on_message(|msg| {
            let visonic = config.visonic.clone();
            let command = msg.payload.to_string();
            async move {
                match visonic.login().await {
                    Ok(visonic) => dispatch_command(command, visonic.clone()).await,
                    Err(err) => {
                        error!("Failed to login to visonic {}", err);
                        None
                    }
                }
            }
        })
        .await;

    Ok(())
}

fn log_unwrap(cmd: String, r: Result<(), VisonicErr>) -> Option<String> {
    return match r {
        Ok(_) => Some(cmd),
        Err(err) => {
            error!("Failure {}: {}", cmd, err);
            Some("ERROR".to_string())
        }
    };
}

async fn dispatch_command(command: String, visonic: AuthedVisonic) -> Option<String> {
    match command {
        s if s.eq("AWAY") => {
            let arm = visonic.arm().await;
            return log_unwrap("AWAY".to_string(), arm);
        }
        s if s.eq("DISARM") => {
            let arm = visonic.disarm().await;
            return log_unwrap("DISARM".to_string(), arm);
        }
        s if s.eq("NIGHT") => {
            let arm = visonic.arm_night().await;
            return log_unwrap("NIGHT".to_string(), arm);
        }
        s if s.eq("STAY") => {
            let arm = visonic.arm_stay().await;
            return log_unwrap("STAY".to_string(), arm);
        }
        s => {
            info!("unknown mqtt command: {}", s.to_string());
            None
        }
    }
}
