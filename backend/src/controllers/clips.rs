use std::ops::Deref;
use std::time::Duration;
use std::time::Instant;

use actix::{Actor, StreamHandler};

//TODO:: Use an appropriate custom error for response type.
use actix_http::ws::HandshakeError;
use actix_web::{web, Error, HttpRequest, HttpResponse};

use actix_http::ws::Item::Continue;
use actix_http::ws::Item::FirstBinary;
use actix_http::ws::Item::FirstText;
use actix_http::ws::Item::Last;

use actix::prelude::*;
use actix_web_actors::ws;

use serde::Deserialize;
use serde::Serialize;

use crate::helpers::errors::ClipError;
use crate::helpers::errors::ClipErrorType;
use crate::helpers::utilities::Utilities;
use crate::services::ffmpeg::FFMpegService;

#[derive(Debug, Clone)]
pub struct ClipDetails {
    pub id: String,
    pub data: usize,
}

#[derive(Debug, Clone)]
struct ClipWS {
    request_type: ClipRequest,
    session_id: String,
    chunks: Vec<u8>,
    clips: Vec<ClipDetails>,
    pub hb: Instant,
}

#[derive(Debug, Clone)]
pub enum ClipRequest {
    Chunk,
    OnPlayback,
    Submission,
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(100);

impl ClipWS {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                // return;
            }

            // ctx.ping(b"");
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipDetailRequest {
    pub id: String,
    pub duration: f64,
}

impl Actor for ClipWS {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClipWS {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                if let ClipRequest::OnPlayback = self.request_type {
                    let clips: Vec<ClipDetailRequest> =
                        serde_json::from_str(text.to_string().as_str()).unwrap();

                    println!("{:?}", clips);
                    let clip_path = FFMpegService::merge_clips(clips, self.session_id.clone());

                    println!("{:?}", clip_path);
                    match clip_path {
                        Ok(_b) => {
                            ctx.text("passed");
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                if let ClipRequest::Chunk = self.request_type {
                    self.chunks.append(&mut bin.to_vec());

                    let file_path = self.session_id.clone();

                    let clip_id = FFMpegService::create_file(file_path, self.chunks.clone());

                    ctx.text(clip_id);
                }
            }
            Ok(ws::Message::Continuation(item)) => {
                if let ClipRequest::Chunk = self.request_type {
                    match item {
                        FirstText(data) => {
                            self.chunks.append(&mut data.to_vec());
                        }
                        FirstBinary(data) => {
                            self.chunks.append(&mut data.to_vec());
                        }
                        Continue(data) => {
                            self.chunks.append(&mut data.to_vec());
                        }
                        Last(data) => {
                            self.chunks.append(&mut data.to_vec());

                            let file_path = self.session_id.clone();

                            let clip_id =
                                FFMpegService::create_file(file_path, self.chunks.clone());

                            ctx.text(clip_id);
                        }
                    }
                }
            }
            Ok(ws::Message::Pong(_t)) => {}
            Ok(ws::Message::Close(_t)) => {
                //Remove temporary session directory.
            }
            Ok(ws::Message::Nop) => {
                println!("{:?}", "nop");
            }
            Err(_err) => (),
        }
    }
}

pub struct ClipController {}

impl ClipController {
    pub fn register_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::resource("/clip/session")
                .route(web::get().to(Self::get_clip_stream))
                .route(
                    web::head().to(|| -> actix_web::HttpResponseBuilder {
                        HttpResponse::MethodNotAllowed()
                    }),
                ),
        );
        cfg.service(
            web::resource("/ws/clips")
                .route(web::get().to(Self::start_clip_session_ws))
                .route(
                    web::head().to(|| -> actix_web::HttpResponseBuilder {
                        HttpResponse::MethodNotAllowed()
                    }),
                ),
        );
        cfg.service(
            web::resource("/ws/clips/submit")
                .route(web::get().to(Self::submit_clip_ws))
                .route(
                    web::head().to(|| -> actix_web::HttpResponseBuilder {
                        HttpResponse::MethodNotAllowed()
                    }),
                ),
        );
    }

    async fn get_clip_stream(req: HttpRequest) -> Result<HttpResponse, ClipError> {
        let cookies = req.cookies().unwrap();

        let session_id: String = Utilities::get_cookie_value(cookies.deref(), "X-FDot-Session");

        if !session_id.is_empty() {
            let stream = FFMpegService::get_session_clip_by_id(session_id).await;
            Ok(stream.into_response(&req))
        } else {
            let mut error = ClipError::from(String::from("Invalid URI"));
            error.set_type(ClipErrorType::InvalidUri);

            Err(error)
        }
    }

    async fn submit_clip_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
        let cookies = req.cookies().unwrap();

        let session_id = Utilities::get_cookie_value(cookies.deref(), "X-FDot-Session");

        if !session_id.is_empty() {
            let session = ws::WsResponseBuilder::new(
                ClipWS {
                    chunks: vec![],
                    session_id,
                    clips: vec![],
                    request_type: ClipRequest::OnPlayback,
                    hb: Instant::now(),
                },
                &req,
                stream,
            )
            .frame_size(10_000_234) //Really important for large streams. Determine Appropriate size.
            .start();
            session
        } else {
            Err(Error::from(HandshakeError::UnsupportedVersion))
        }
    }
    async fn start_clip_session_ws(
        req: HttpRequest,
        stream: web::Payload,
    ) -> Result<HttpResponse, Error> {
        let cookies = req.cookies().unwrap();

        let session_id = Utilities::get_cookie_value(cookies.deref(), "X-FDot-Session");

        if !session_id.is_empty() {
            let session = ws::WsResponseBuilder::new(
                ClipWS {
                    chunks: vec![],
                    session_id,
                    clips: vec![],
                    request_type: ClipRequest::Chunk,
                    hb: Instant::now(),
                },
                &req,
                stream,
            )
            .frame_size(10_000_234) //Really important for large streams. Determine Appropriate size.
            .start();
            session
        } else {
            Err(Error::from(HandshakeError::UnsupportedVersion))
        }
    }
}
