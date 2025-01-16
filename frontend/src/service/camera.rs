use gloo_net::websocket::Message;

use futures::{SinkExt, StreamExt};
use std::collections::HashSet;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::Blob;

use yew_agent::{Agent, AgentLink, Context, HandlerId};

use crate::models::{
    CameraContext, CameraContextAction, ClipDetailRequest, ClipDetails, DeviceError, DeviceType,
    Msg, Request,
};

use super::web_socket::WebSocketService;

pub struct CameraService {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for CameraService {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = Request;
    type Output = CameraContext;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, _msg: Self::Input, _id: HandlerId) {
        let mut has_sent_to_subs = false;

        let mut context = CameraContext {
            context_type: None,
            stream: None,
            recorder: None,
            chunk: None,
            devices: None,
            device_error: None,
            clip_details: None,
            merged_clip: None,
        };

        match _msg {
            Request::SendBlobChunk(chunk) => {
                context.context_type = Some(CameraContextAction::SendBlobChunk);
                context.chunk = Some(chunk);

                let subs = self.subscribers.clone();
                let link = self.link.clone();
                let mut _context = context.clone();

                for sub in subs.iter().filter(|s| s.is_respondable()) {
                    link.respond(*sub, _context.clone());
                }

                has_sent_to_subs = true;

                if let Some(ws) = WebSocketService::public("clips") {
                    let (mut write, mut read) = ws.context.split();
                    let _buffer = JsFuture::from(context.chunk.clone().unwrap().array_buffer());

                    spawn_local(async move {
                        let buffer = _buffer.await;

                        let buffer_array = js_sys::Uint8Array::new(&buffer.unwrap());
                        let bytes = buffer_array.to_vec();

                        write.send(Message::Bytes(bytes)).await.unwrap();
                    });

                    spawn_local(async move {
                        while let Some(msg) = read.next().await {
                            let message_type = match msg {
                                Ok(it) => it,
                                _ => continue,
                            };

                            if let Message::Text(clip_id) = message_type {
                                _context.context_type = Some(CameraContextAction::AddedClip);
                                _context.clip_details = Some(ClipDetails {
                                    id: clip_id,
                                    duration: 0.0,
                                    chunk: Blob::new().unwrap(),
                                });

                                for sub in subs.iter().filter(|s| s.is_respondable()) {
                                    link.respond(*sub, _context.clone());
                                }
                            }
                        }
                    });
                };
            }
            Request::OnSubmission() => {}
            Request::SendDeviceList(devices) => {
                context.context_type = Some(CameraContextAction::SendDeviceList);
                context.devices = Some(devices);
            }
            Request::SendUserMedia(stream) => {
                context.context_type = Some(CameraContextAction::SendUserMedia);
                context.stream = Some(stream);
            }
            Request::SendMicFrequency() => {
                context.context_type = Some(CameraContextAction::SendMicFrequency);
            }
            Request::SendDeviceError(value) => {
                //Because theres no other way to parse this error.
                let mut string_indexer = format!("{:?}", value);
                string_indexer = string_indexer.replace("JsValue(", "");
                string_indexer = string_indexer.replace(')', "");
                string_indexer = string_indexer.replace('\n', " ");
                string_indexer = string_indexer.replace("undefined", "");

                let mut error = DeviceError {
                    device_type: DeviceType::Camera,
                    valid: false,
                    message: string_indexer.clone(),
                };

                if string_indexer.contains("audio") {
                    error.device_type = DeviceType::Microphone;
                }

                context.context_type = Some(CameraContextAction::SendDeviceError);
                context.device_error = Some(error);
            }
            Request::SendClipDetails(clip_details) => {
                context.context_type = Some(CameraContextAction::SendClipDetails);
                context.clip_details = Some(clip_details);
            }
            Request::OnPlayback(clips) => {
                if let Some(ws) = WebSocketService::public("clips/submit") {
                    let (mut write, mut read) = ws.context.split();

                    let subs = self.subscribers.clone();
                    let link = self.link.clone();
                    let mut _context = context.clone();

                    spawn_local(async move {
                        let req = serde_json::to_string::<Vec<ClipDetailRequest>>(&clips).unwrap();
                        write.send(Message::Text(req)).await.unwrap();
                    });
                    spawn_local(async move {
                        while let Some(msg) = read.next().await {
                            let message_type = match msg {
                                Ok(it) => it,
                                _ => continue,
                            };

                            if let Message::Text(_clip_data) = message_type {
                                _context.context_type = Some(CameraContextAction::MergedClip);
                                _context.merged_clip = None;

                                for sub in subs.iter().filter(|s| s.is_respondable()) {
                                    link.respond(*sub, _context.clone());
                                }
                            }
                        }
                    });
                }
                has_sent_to_subs = true;
            }
        }

        if !has_sent_to_subs {
            for sub in self.subscribers.iter().filter(|s| s.is_respondable()) {
                self.link.respond(*sub, context.clone());
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn destroy(&mut self) {}

    fn name_of_resource() -> &'static str {
        "main.js"
    }

    fn resource_path_is_relative() -> bool {
        false
    }

    fn is_module() -> bool {
        false
    }
}
