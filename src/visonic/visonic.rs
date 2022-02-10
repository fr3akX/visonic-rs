use std::fmt::{Display, Formatter};
use std::future::Future;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::visonic::*;

#[derive(Clone, Deserialize)]
pub struct Visonic {
    pub hostname: String,
    pub user_code: String,
    pub app_id: String,
    pub partition: i8,
    pub user_email: String,
    pub user_password: String,
    pub panel_id: String,
}

#[derive(Clone)]
pub struct AuthedVisonic {
    pub(crate) visonic: Visonic,
    pub(crate) user_token: String,
    pub(crate) session_token: String,
}

#[derive(Serialize)]
struct ReqLogin {
    email: String,
    password: String,
    app_id: String,
}

#[derive(Deserialize)]
pub struct RespLogin {
    pub user_token: String,
}

#[derive(Serialize)]
pub struct ReqPanelLogin {
    user_code: String,
    app_type: String,
    app_id: String,
    panel_serial: String,
}

#[derive(Deserialize)]
pub struct ResPanelLogin {
    pub session_token: String,
}

#[derive(Deserialize)]
pub struct RespVersion {
    pub rest_versions: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum VisonicErr {
    VersionNotSupported(String),
    HttpError(u16, String),
    RetriesExhausted,
}

impl From<reqwest::Error> for VisonicErr {
    fn from(err: reqwest::Error) -> Self {
        let status = err.status().map_or_else(|| 0, |status| status.as_u16());
        let msg = err.to_string();
        VisonicErr::HttpError(status, msg)
    }
}

impl Display for VisonicErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VisonicErr::VersionNotSupported(vers) => {
                write!(f, "VisonicErr::VersionNotSupported({})", vers)
            }
            VisonicErr::HttpError(code, s) => write!(f, "VisonicErr::HttpError({}, {})", code, s),
            VisonicErr::RetriesExhausted => write!(f, "VisonicErr::RetriesExhausted"),
        }
    }
}

impl Visonic {
    async fn version(&self) -> Result<RespVersion, VisonicErr> {
        let ep = format!("https://{}/rest_api{}", &self.hostname, RES_VERSIONS);
        let res: RespVersion = reqwest::get(ep).await?.json().await?;

        Ok(res)
    }

    pub async fn check_ver(&self) -> Result<(), VisonicErr> {
        match self.version().await {
            Ok(r) => r
                .rest_versions
                .iter()
                .find(|s| s.eq(&&REST_VERSION.to_string()))
                .map_or_else(
                    || {
                        Err(VisonicErr::VersionNotSupported(format!(
                            "{:?}",
                            r.rest_versions
                        )))
                    },
                    |_| Ok(()),
                ),
            Err(e) => Err(e),
        }
    }

    async fn panel_login(&self, user_code: String) -> Result<ResPanelLogin, VisonicErr> {
        let req = ReqPanelLogin {
            user_code: self.user_code.to_string(),
            app_type: APP_TYPE.to_string(),
            app_id: self.app_id.to_string(),
            panel_serial: self.panel_id.to_string(),
        };

        let resp: ResPanelLogin = reqwest::Client::new()
            .post(uri(&self.hostname, RES_PANEL_LOGIN))
            .header("User-Token", user_code)
            .json(&req)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }
    async fn account_login(&self) -> Result<RespLogin, VisonicErr> {
        let req = ReqLogin {
            email: self.user_email.to_string(),
            password: self.user_password.to_string(),
            app_id: self.app_id.to_string(),
        };

        let resp: RespLogin = reqwest::Client::new()
            .post(uri(&self.hostname, RES_AUTH))
            .json(&req)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }

    pub async fn login(&self) -> Result<AuthedVisonic, VisonicErr> {
        self.check_ver().await?;
        let login = self.account_login().await?;
        let session = self.panel_login(login.user_token.to_string()).await?;

        Ok(AuthedVisonic {
            visonic: self.clone(),
            session_token: session.session_token,
            user_token: login.user_token,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct Partition {
    pub id: u16,
    pub state: String,
    pub status: String,
    pub ready: bool,
}

#[derive(Deserialize)]
pub struct ResStatus {
    pub connected: bool,
    pub partitions: Vec<Partition>,
}

#[derive(Serialize)]
struct ReqSetState {
    partition: i16,
    state: String,
}

#[derive(Deserialize, Clone)]
pub struct ResProcessToken {
    pub process_token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResProcessStatus {
    pub token: String,
    pub status: String,
    pub error: Option<String>,
}

impl AuthedVisonic {
    pub async fn status(&self) -> Result<ResStatus, VisonicErr> {
        self.get_json::<ResStatus>(RES_STATUS).await
    }

    pub async fn status_txt(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_STATUS).await
    }

    pub async fn arm(&self) -> Result<(), VisonicErr> {
        let res = self.set_status("AWAY".to_string()).await?;
        let _ = self.process_status(res).await?;
        Ok(())
    }

    pub async fn disarm(&self) -> Result<(), VisonicErr> {
        let res = self.set_status("DISARM".to_string()).await?;
        let _ = self.process_status(res).await?;
        Ok(())
    }

    pub async fn arm_night(&self) -> Result<(), VisonicErr> {
        let res = self.set_status("NIGHT".to_string()).await?;
        let _ = self.process_status(res).await?;
        Ok(())
    }

    pub async fn arm_stay(&self) -> Result<(), VisonicErr> {
        let res = self.set_status("STAY".to_string()).await?;
        let _ = self.process_status(res).await?;
        Ok(())
    }

    async fn set_status(&self, status: String) -> Result<ResProcessToken, VisonicErr> {
        let req = ReqSetState {
            partition: -1,
            state: status,
        };
        let res = reqwest::Client::new()
            .post(uri(&self.visonic.hostname, RES_SET_STATE))
            .json(&req)
            .with_user_session_token(self.user_token.to_string(), self.session_token.to_string())
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }
    pub async fn process_status(
        &self,
        token: ResProcessToken,
    ) -> Result<Vec<ResProcessStatus>, VisonicErr> {
        let res = self.execute_while(
            || self.process_status_once(token.clone()),
            |result| {
                result
                    .iter()
                    .find(|item| item.status.eq("succeeded"))
                    .is_some()
            },
            5,
        );

        res.await
    }
    async fn process_status_once(
        &self,
        token: ResProcessToken,
    ) -> Result<Vec<ResProcessStatus>, VisonicErr> {
        let url = format!(
            "{}?process_tokens={}",
            uri(&self.visonic.hostname, RES_PROCESS_STATUS),
            token.process_token
        );

        let res: Vec<ResProcessStatus> = reqwest::Client::new()
            .get(url)
            .with_user_session_token(self.user_token.to_string(), self.session_token.to_string())
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }
    async fn execute_while<F, R: Clone, Fut, P>(
        &self,
        f: F,
        predicate: P,
        limit: u8,
    ) -> Result<R, VisonicErr>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<R, VisonicErr>>,
        P: Fn(&R) -> bool,
    {
        let sleep = tokio::time::sleep(Duration::from_secs(1));
        tokio::pin!(sleep);

        for _ in 1..limit {
            let r = f().await;
            match r {
                Ok(r) if (predicate(&r)) => return Ok(r.clone()),
                Ok(_) => (),
                Err(_) => (),
            }

            tokio::select! {
                () = &mut sleep => {
                    sleep.as_mut().reset(Instant::now() + Duration::from_secs(1));
                },
            }
        }

        Err(VisonicErr::RetriesExhausted)
    }

    //TODO: NEED SAMPLE
    pub async fn events(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_EVENTS).await
    }

    //TODO: NEED SAMPLE
    pub async fn alarms(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_ALARMS).await
    }

    //TODO: NEED SAMPLE
    pub async fn alerts(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_ALERTS).await
    }

    //TODO: NEED SAMPLE
    pub async fn troubles(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_TROUBLES).await
    }

    //TODO: NEED SAMPLE
    pub async fn panel_info(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_PANEL_INFO).await
    }

    pub async fn wakeup_sms(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_WAKEUP_SMS).await
    }

    pub async fn devices(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_DEVICES).await
    }

    pub async fn locations(&self) -> Result<String, VisonicErr> {
        self.get_text(RES_LOCATIONS).await
    }

    async fn get_text(&self, endpoint: &str) -> Result<String, VisonicErr> {
        let s: String = reqwest::Client::new()
            .get(uri(&self.visonic.hostname, endpoint))
            .with_user_session_token(self.user_token.to_string(), self.session_token.to_string())
            .send()
            .await?
            .text()
            .await?;

        Ok(s)
    }

    async fn get_json<R: DeserializeOwned>(&self, endpoint: &str) -> Result<R, VisonicErr> {
        let res: R = reqwest::Client::new()
            .get(uri(&self.visonic.hostname, endpoint))
            .with_user_session_token(self.user_token.to_string(), self.session_token.to_string())
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }
}
