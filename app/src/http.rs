use gtk::glib::{self, Bytes, ValueDelegate};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as JsonObject;
use soup::{prelude::*, Message, Status};

fn endpoint_to_uri(endpoint: &str) -> String {
    const SERVER_URI: &str = "http://localhost:3000";

    let separator = if endpoint.starts_with('/') { "" } else { "/" };

    format!("{SERVER_URI}{separator}{endpoint}")
}

#[derive(ValueDelegate, Default, Debug)]
pub struct Session(soup::Session);

impl Session {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(soup::Session::new())
    }

    pub async fn post<Res: DeserializeOwned>(
        &self,
        request: impl Serialize,
        endpoint: &str,
    ) -> Result<Res, Error> {
        // FIXME: Handle deserialization and response errors properly

        let uri = endpoint_to_uri(endpoint);
        let msg = Message::new("POST", &uri).unwrap_or_else(|err| {
            panic!("post: '{endpoint}' does not make a valid URI (derived URI: '{uri}'): {err}")
        });

        let serialized_request = serde_json::to_string(&request).unwrap();
        let bytes = Bytes::from_owned(serialized_request);
        msg.set_request_body_from_bytes(Some("application/json"), Some(&bytes));

        let raw_response = self
            .0
            .send_and_read_future(&msg, glib::Priority::DEFAULT)
            .await
            .map_err(Error::Network)?;

        if msg.status_code() >= 400 {
            let json_object: JsonObject =
                serde_json::from_slice(&raw_response).map_err(Error::Deserialization)?;
            Err(Error::Api {
                status: msg.status(),
                status_code: msg.status_code(),
                msg: match json_object.as_object() {
                    Some(object) => object
                        .get("error")
                        .map(|msg| msg.to_string())
                        .unwrap_or_else(|| "Unknown error".to_string()),
                    _ => json_object.to_string(),
                },
            })
        } else {
            Ok(serde_json::from_slice(&raw_response).map_err(Error::Deserialization)?)
        }
    }

    pub async fn get<Res: DeserializeOwned>(&self, endpoint: &str) -> Result<Res, Error> {
        let uri = endpoint_to_uri(endpoint);
        let msg = Message::new("GET", &uri).unwrap_or_else(|err| {
            panic!("post: '{endpoint}' does not make a valid URI (derived URI: '{uri}'): {err}")
        });

        let raw_response = self
            .0
            .send_and_read_future(&msg, glib::Priority::DEFAULT)
            .await
            .map_err(Error::Network)?;

        if msg.status_code() >= 400 {
            let json_object: JsonObject =
                serde_json::from_slice(&raw_response).map_err(Error::Deserialization)?;
            Err(Error::Api {
                status: msg.status(),
                status_code: msg.status_code(),
                msg: match json_object.as_object() {
                    Some(object) => object
                        .get("error")
                        .map(|msg| msg.to_string())
                        .unwrap_or_else(|| "Unknown error".to_string()),
                    _ => json_object.to_string(),
                },
            })
        } else {
            Ok(serde_json::from_slice(&raw_response).map_err(Error::Deserialization)?)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("A network error occured: {0}")]
    Network(glib::Error),
    #[error("The server returned an error: '{msg}' status: {status_code} ({status})")]
    Api {
        status: Status,
        status_code: u32,
        msg: String,
    },
    #[error("Encountered a deserialization error: {0}")]
    Deserialization(serde_json::Error),
}

#[derive(glib::Boxed, Clone, Debug, Default)]
#[boxed_type(name = "LibSessionCookie", nullable)]
pub struct SessionCookie {
    cookie: schema::session::Cookie,
    user_type: i64,
}

impl SessionCookie {
    pub fn new(id: i64, password: String, user_type: i64) -> Self {
        Self {
            cookie: schema::session::Cookie { id, password },
            user_type,
        }
    }

    pub fn cookie(&self) -> &schema::session::Cookie {
        &self.cookie
    }

    pub fn user_type(&self) -> i64 {
        self.user_type
    }
}
