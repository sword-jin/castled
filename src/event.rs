use std::vec;

use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tonic::Status;

/// ClientEvent is used to communicate between the control server and data server.
/// When the control server receives a client request, eventually it will
/// send a ClientEvent to the data server if no error occurs.
pub struct ClientEvent {
    // the payload of the event.
    pub payload: Payload,
    // the data server will send back the status of the control server
    // after it handles this event.
    pub resp: oneshot::Sender<ClientEventResponse>,
    // when client exits, the control server will cancel the listener.
    pub close_listener: CancellationToken,
    // data server sends events to this channel continuously.
    pub incoming_events: IncomingEventSender,
}

/// ClientEventResponse is the response of the ClientEvent.
#[derive(Debug)]
pub enum ClientEventResponse {
    Registered {
        /// status is not None if the registration failed.
        status: Option<Status>,
        // entrypoint is available if the registration is successful.
        entrypoint: Vec<String>,
    },
}

impl ClientEventResponse {
    pub(crate) fn registered(entrypoint: Vec<String>) -> Self {
        Self::Registered {
            status: None,
            entrypoint,
        }
    }

    pub(crate) fn registered_failed(status: Status) -> Self {
        Self::Registered {
            status: Some(status),
            entrypoint: vec![],
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_registered(&self) -> bool {
        matches!(self, Self::Registered { .. })
    }

    #[allow(dead_code)]
    pub(crate) fn entrypoint(&self) -> Option<&Vec<String>> {
        match self {
            Self::Registered { entrypoint, .. } => Some(entrypoint),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn status(&self) -> Option<&Status> {
        match self {
            Self::Registered { status, .. } => status.as_ref(),
        }
    }
}

/// Payload is the data of the ClientEvent.
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Payload {
    RegisterTcp {
        port: u16,
    },
    RegisterUdp {
        port: u16,
    },
    // RegisterHttp is used to notify the server to register a http tunnel.
    // must provide one of the following fields: port, subdomain, domain.
    RegisterHttp {
        port: u16,
        subdomain: Bytes,
        domain: Bytes,
        random_subdomain: bool,
    },
}

/// IncomingEventSender is used to notify the server to add | remove to the connection list.
pub type IncomingEventSender = mpsc::Sender<UserIncoming>;

/// When the data server receives a user request (generally from the browser or terminal),
/// when it will send `Add` to send a bridge with the control server.
pub enum UserIncoming {
    Add(crate::bridge::IdDataSenderBridge),
    Remove(Bytes),
}
