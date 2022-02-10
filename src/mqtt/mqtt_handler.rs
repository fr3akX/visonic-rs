use log::error;
use rumqttc::{AsyncClient, ClientError, Event, EventLoop, Incoming, LastWill, MqttOptions, QoS};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::future::Future;
use tokio::task;
use tokio::task::{JoinError, JoinHandle};

const LWT_OFFLINE: &str = "OFFLINE";
const LWT_ONLINE: &str = "ONLINE";

#[derive(Clone, Deserialize)]
pub struct MqttHandlerConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub command_topic: String,
    pub status_topic: String,
    pub info_topic: String,
    pub lwt_topic: String,
}

pub struct MqttAsyncConnection {
    config: MqttHandlerConfig,
    client: AsyncClient,
    connection: EventLoop,
}

impl MqttAsyncConnection {
    pub async fn publish(&self, topic: String, payload: String) -> Result<(), ClientError> {
        self.client
            .publish(topic, QoS::AtLeastOnce, true, payload)
            .await
    }

    pub async fn on_message<F, Fut>(&mut self, handler: F)
    where
        F: Fn(Message) -> Fut,
        Fut: Future<Output = Option<String>>,
    {
        loop {
            let event = &self.connection.poll().await.unwrap(); //its ok to fail here
            match event {
                Event::Incoming(Incoming::Publish(p)) => {
                    let r = std::str::from_utf8(&p.payload).map(|s| Message {
                        topic: p.topic.to_string(),
                        payload: s.to_string(),
                    });

                    match r {
                        Ok(msg) => match handler(msg).await {
                            Some(msg) => {
                                let pub_result = &self
                                    .client
                                    .publish(&self.config.status_topic, QoS::AtLeastOnce, true, msg)
                                    .await;

                                match pub_result {
                                    Ok(_) => (),
                                    Err(err) => error!("Error publishing to mqtt: {}", err),
                                }
                            }
                            None => (),
                        },
                        Err(err) => error!("Failed to decode MQTT message: {}", err),
                    }
                }
                _ => async { () }.await,
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    Mqtt(ClientError),
    System(JoinError),
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::Mqtt(err) => write!(f, "HandlerError::Mqtt {}", err),
            HandlerError::System(join) => write!(f, "HandlerError::System {}", join),
        }
    }
}
impl From<ClientError> for HandlerError {
    fn from(err: ClientError) -> Self {
        Self::Mqtt(err)
    }
}

impl MqttHandlerConfig {
    pub async fn connect(&self) -> Result<MqttAsyncConnection, HandlerError> {
        async fn do_subscribe(
            client: AsyncClient,
            eventloop: EventLoop,
            config: MqttHandlerConfig,
        ) -> Result<(AsyncClient, EventLoop), ClientError> {
            match client
                .subscribe(config.command_topic, QoS::ExactlyOnce)
                .await
            {
                Ok(_) => Ok((client, eventloop)),
                Err(e) => Err(e),
            }
        }

        let mut mqttoptions = MqttOptions::new(&self.id, &self.host, self.port);
        mqttoptions.set_credentials(&self.username, &self.password);
        mqttoptions.set_last_will(LastWill::new(
            &self.lwt_topic,
            LWT_OFFLINE,
            QoS::AtLeastOnce,
            true,
        ));

        let (client, connection) = AsyncClient::new(mqttoptions, 10);

        client
            .publish(&self.lwt_topic, QoS::AtLeastOnce, true, LWT_ONLINE)
            .await?;

        let conf = self.clone();
        let x: JoinHandle<Result<(AsyncClient, EventLoop), ClientError>> =
            task::spawn(async move { do_subscribe(client, connection, conf.clone()).await });

        match x.await {
            Ok(join) => match join {
                Ok(r) => Ok(MqttAsyncConnection {
                    connection: r.1,
                    client: r.0,
                    config: self.clone(),
                }),
                Err(e) => Err(HandlerError::Mqtt(e)),
            },
            Err(e) => Err(HandlerError::System(e)),
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.topic, self.payload)
    }
}
