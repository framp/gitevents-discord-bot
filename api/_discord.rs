use crate::_error::Error;
use ed25519_dalek::{PublicKey, Signature, Verifier, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use http::{Response, StatusCode};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use vercel_lambda::{IntoResponse, Request};

#[derive(FromPrimitive)]
pub enum InteractionRequestType {
    Ping = 1,
    ApplicationCommand = 2,
    ModalSubmit = 5,
}

pub enum CommandResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    Modal = 9,
}
pub enum CommandResponseFlag {
    Ephemeral = 64,
}

pub enum MessageStyle {
    Short = 1,
    Long = 2,
}

pub enum CommandRequest {
    Ping,
    NewEvent,
    ModalSubmit(String, String, String, String, String, String),
}

impl<'de> Deserialize<'de> for CommandRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CommandVisitor;

        impl<'de> serde::de::Visitor<'de> for CommandVisitor {
            type Value = CommandRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer which represent a Discord interaction")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut type_field = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "type" {
                        type_field = Some(map.next_value::<i64>()?);
                    } else {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }

                let type_value =
                    type_field.ok_or_else(|| serde::de::Error::missing_field("type"))?;

                match FromPrimitive::from_i64(type_value) {
                    Some(InteractionRequestType::Ping) => Ok(CommandRequest::Ping),
                    Some(InteractionRequestType::ApplicationCommand) => {
                        Ok(CommandRequest::NewEvent)
                    }
                    Some(InteractionRequestType::ModalSubmit) => Ok(CommandRequest::ModalSubmit(
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                    )),
                    _ => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Signed(type_value),
                        &"an integer which represent a Discord interaction",
                    )),
                }
            }
        }

        let visitor = CommandVisitor;
        deserializer.deserialize_map(visitor)
    }
}

#[derive(Serialize, Debug)]
pub enum CommandResponse {
    Pong,
    Modal,
    EventSuccess(String),
    EventFail,
}

impl IntoResponse for CommandResponse {
    fn into_response(self) -> http::Response<vercel_lambda::Body> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/json")
            .body(vercel_lambda::Body::from(match self {
                CommandResponse::Pong => get_pong_json().to_string(),
                CommandResponse::Modal => get_modal_json().to_string(),
                CommandResponse::EventSuccess(link) => get_message_success_json(&link).to_string(),
                CommandResponse::EventFail => get_message_fail_json().to_string(),
            }))
            .expect("Internal Server Error")
    }
}

fn get_modal_component_json(
    id: &str,
    label: &str,
    placeholder: &str,
    style: MessageStyle,
) -> Value {
    json!({
        "type": 1,
        "components": [{
            "type": 4,
            "custom_id": id,
            "label": label,
            "style": style as u8,
            "min_length": 1,
            "max_length": 100,
            "placeholder": placeholder,
            "required": true
        }]
    })
}

fn get_modal_json() -> Value {
    json!({
        "type": CommandResponseType::Modal as u8,
        "data": {
            "title": "New Event",
            "custom_id": "new_event",
            "components": [
                get_modal_component_json("name", "Name", "Event name", MessageStyle::Short),
                get_modal_component_json("description", "Description", "A concise description", MessageStyle::Long),
                get_modal_component_json("location", "Location", "online", MessageStyle::Short),
                get_modal_component_json("date", "Date", "15/12/2022", MessageStyle::Short),
                get_modal_component_json("time", "Time", "12:30pm", MessageStyle::Short),
                get_modal_component_json("duration", "Duration", "1h30m", MessageStyle::Short),
            ]
        }
    })
}

fn get_message_success_json(link: &str) -> Value {
    json!({
        "type": CommandResponseType::ChannelMessageWithSource as u8,
        "data": {
            "content": format!("An event was just created: {}", link)
        }
    })
}

fn get_message_fail_json() -> Value {
    json!({
        "type": CommandResponseType::ChannelMessageWithSource as u8,
        "data": {
            "content": "There was an error creating your event",
            "flags": CommandResponseFlag::Ephemeral as u8
        }
    })
}

fn get_pong_json() -> Value {
    json!({ "type": CommandResponseType::Pong as u8 })
}

pub fn handle_commands(req: &Request) -> Result<CommandResponse, Error> {
    let body: CommandRequest = serde_json::from_slice(req.body())?;

    Ok(match body {
        CommandRequest::Ping => CommandResponse::Pong,
        CommandRequest::NewEvent => CommandResponse::Modal,
        CommandRequest::ModalSubmit(name, description, location, date, time, duration) => {
            println!("do something with github");
            CommandResponse::EventFail
        }
    })
}

pub fn validate_headers(req: &Request, public_key: &str) -> Result<(), Error> {
    let sig = req.headers().get("x-signature-ed25519");
    let timestamp = req.headers().get("x-signature-timestamp");
    if let (Some(sig), Some(timestamp)) = (sig, timestamp) {
        let public_key_hex = hex::decode(public_key)?;
        let signature_hex = hex::decode(sig)?;
        let public_key = PublicKey::from_bytes(&public_key_hex.as_slice()[..PUBLIC_KEY_LENGTH])?;
        let signature = Signature::from_bytes(&signature_hex.as_slice()[..SIGNATURE_LENGTH])?;
        let mut full_body = Vec::from(timestamp.as_bytes());
        full_body.extend_from_slice(req.body());
        public_key.verify(full_body.as_slice(), &signature)?;
        Ok(())
    } else {
        Err(Error::InvalidInput(
            "You need to provide both signature and timestamp".to_string(),
        ))
    }
}

pub async fn create_command(application_id: &str, bot_token: &str) -> Result<(), Error> {
    let client = Client::new();
    let url = format!(
        "https://discord.com/api/v10/applications/{}/commands",
        application_id
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bot {}", bot_token)).unwrap(),
    );

    let response = client
        .post(url)
        .headers(headers)
        .body(
            json!({
                "name": "new_event".to_string(),
                "type_value": 1,
                "description": "Create a new event on GitEvents".to_string(),
            })
            .to_string(),
        )
        .send()
        .await?;

    println!("{:?}", response);
    Ok(())
}
