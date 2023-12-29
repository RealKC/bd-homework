use gtk::glib::{self, Bytes, ValueDelegate};
use serde::{de::DeserializeOwned, Serialize};
use soup::{prelude::*, Message};

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
    ) -> Result<Res, glib::Error> {
        // FIXME: Handle deserialization and response errors properly

        let uri = endpoint_to_uri(endpoint);
        let msg = Message::new("POST", &uri)
            .unwrap_or_else(|err| panic!("post: '{endpoint}' does not make a valid URI: {err}"));

        let serialized_request = serde_json::to_string(&request).unwrap();
        let bytes = Bytes::from_owned(serialized_request);
        msg.set_request_body_from_bytes(Some("application/json"), Some(&bytes));

        let raw_response = self
            .0
            .send_and_read_future(&msg, glib::Priority::DEFAULT)
            .await?;

        Ok(serde_json::from_slice(&raw_response).unwrap())
    }
}
