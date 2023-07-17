use reqwest::RequestBuilder;

pub(crate) const APP_TYPE: &str = "com.visonic.PowerMaxApp";
// const UA: &str = "Visonic%20GO/2.8.62.91 CFNetwork/901.1 Darwin/17.6.0";
pub(crate) const REST_VERSION: &str = "10.0";

pub(crate) const RES_PANEL_LOGIN: &str = "/panel/login";
pub(crate) const RES_AUTH: &str = "/auth";
pub(crate) const RES_STATUS: &str = "/status";
pub(crate) const RES_VERSIONS: &str = "/version";
pub(crate) const RES_SET_STATE: &str = "/set_state";
pub(crate) const RES_PROCESS_STATUS: &str = "/process_status";
pub(crate) const RES_EVENTS: &str = "/events";
pub(crate) const RES_ALARMS: &str = "/alarms";
pub(crate) const RES_ALERTS: &str = "/alerts";
pub(crate) const RES_TROUBLES: &str = "/troubles";
pub(crate) const RES_PANEL_INFO: &str = "/panel_info";
pub(crate) const RES_WAKEUP_SMS: &str = "/wakeup_sms";
pub(crate) const RES_DEVICES: &str = "/devices";
pub(crate) const RES_LOCATIONS: &str = "/locations";

pub(crate) fn uri(hostname: &String, endpoint: &str) -> String {
    format!("https://{}/rest_api/{}{}", hostname, REST_VERSION, endpoint)
}

pub(crate) trait RequestBuilderExt {
    fn with_user_session_token(self, user_token: String, session_token: String) -> Self;
    fn with_user_token(self, user_token: Option<String>) -> Self;
    fn with_session_token(self, session_token: Option<String>) -> Self;
}
impl RequestBuilderExt for RequestBuilder {
    fn with_user_session_token(self, user_token: String, session_token: String) -> Self {
        self.with_user_token(Some(user_token))
            .with_session_token(Some(session_token))
    }
    fn with_user_token(self, user_token: Option<String>) -> Self {
        match user_token {
            Option::Some(user_token) => self.header("User-Token", user_token.to_string()),
            Option::None => self,
        }
    }
    fn with_session_token(self, session_token: Option<String>) -> Self {
        match session_token {
            Option::Some(session_token) => self.header("Session-Token", session_token.to_string()),
            Option::None => self,
        }
    }
}
