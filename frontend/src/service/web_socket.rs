use crate::utilities::Utilities;
use gloo_net::websocket::futures::WebSocket;

pub struct WebSocketService {
    pub context: WebSocket,
}

impl WebSocketService {
    pub fn public(path: &'static str) -> Option<WebSocketService> {
        let mut url = Utilities::config("ws_url");
        url.push_str(path);

        match WebSocket::open(url.as_str()) {
            Ok(context) => Some(WebSocketService { context }),
            Err(_) => None,
        }
    }
}
