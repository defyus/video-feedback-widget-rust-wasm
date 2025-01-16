use gloo_timers::callback::Interval;

use js_sys::Array;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;

use web_sys::{
    window, AnalyserNode, AudioContext, Blob, BlobEvent, BlobPropertyBag, HtmlMediaElement,
    HtmlVideoElement, MediaDeviceInfo, MediaDeviceKind, MediaRecorder, MediaRecorderOptions,
    MediaStream, MediaStreamConstraints,
};

use yew::prelude::*;

use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

use super::loading_animated::Loading;

use crate::form::FormBuilder;
use crate::models::{
    ActiveDevices, CameraContext, CameraContextAction, CameraView, ClipDetailRequest, ClipDetails,
    ConstraintDeviceId, ConstraintOptions, DeviceError, DeviceType, FieldValue, Request,
};

use crate::service::camera::CameraService;
use crate::utilities::Utilities;

pub struct Camera {
    view: CameraView,

    camera_id: String,
    is_camera_preview_active: bool,

    stream: MediaStream,
    recorder: MediaRecorder,

    duration: f64,
    preview_duration: f64,

    current_timestamp: f64,

    timestamp: f64,
    preview_timestamp: f64,

    timestamp_timer: Interval,
    preview_timestamp_timer: Interval,

    video_ouput_error: DeviceError,
    audio_input_error: DeviceError,

    video_element: HtmlMediaElement,

    audio_context: AudioContext,
    audio_analyser: AnalyserNode,
    audio_average_percent: u64,
    audio_interval_timer: Interval,

    devices: Vec<MediaDeviceInfo>,

    chunks: Array,
    clips: Vec<ClipDetails>,

    active_devices: ActiveDevices,

    discard_hover_state: bool,
    last_discarded_clip: Option<ClipDetails>,

    _cs: Dispatcher<CameraService>,
    producer: Box<dyn Bridge<CameraService>>,

    is_recording: bool,
    is_playing: bool,
    is_mute: bool,
}

pub enum Msg {
    SetStreamRecorder(MediaStream, MediaRecorder),
    CameraServiceMessenger(CameraContext),
    StartRecording(),
    StopRecording(),
    SetView(CameraView),
    SetDevice(Event, DeviceType),
    SetDeviceID(String, DeviceType),
    ToggleCameraPreview(),
    DisplaySettings(),
    DisplayEditor(),
    DiscardHoverState(bool),
    OnDiscardClick(),
    OnClipUndo(),
    PlaySavedClips(),
    PreviewOnPlayToggle(),
    PreviewOnMuteToggle(),
    PreviewTimestamp(),
    Timestamp(),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub duration: f64,
    pub view: CameraView,
}

impl Component for Camera {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let document = window().unwrap().document().unwrap();

        let mut camera_id = String::from("camera-");
        camera_id.push_str(Utilities::rnd_id("").as_str());

        let mut vid_id = String::from("vid-");
        vid_id.push_str(camera_id.as_str());

        let video_element = document
            .create_element("video")
            .unwrap()
            .dyn_into::<HtmlMediaElement>()
            .unwrap();
        video_element.set_attribute("id", &vid_id).unwrap_throw();

        let stream = MediaStream::new().unwrap();

        let audio_context = AudioContext::new().unwrap();
        let audio_analyser = audio_context.create_analyser().unwrap();
        let current_view = &ctx.props().view;

        let mut camera = Self {
            view: current_view.clone(),

            _cs: CameraService::dispatcher(),
            producer: CameraService::bridge(ctx.link().callback(Msg::CameraServiceMessenger)),

            camera_id,
            is_camera_preview_active: false,

            stream: stream.clone(),
            video_element,
            recorder: MediaRecorder::new_with_media_stream(&stream).unwrap(),

            current_timestamp: 0.0,

            duration: ctx.props().duration.clone(),
            preview_duration: 0.0,

            timestamp: 0.0,
            preview_timestamp: 0.0,

            timestamp_timer: Interval::new(1000, || {}),
            preview_timestamp_timer: Interval::new(1000, || {}),

            video_ouput_error: DeviceError {
                device_type: DeviceType::Camera,
                message: "".to_string(),
                valid: false,
            },
            audio_input_error: DeviceError {
                device_type: DeviceType::Microphone,
                message: "".to_string(),
                valid: false,
            },

            audio_context,
            audio_analyser,
            audio_average_percent: 0,
            audio_interval_timer: Interval::new(1000, || {}),

            chunks: Array::new(),
            clips: vec![],

            devices: vec![],

            active_devices: ActiveDevices {
                camera: None,
                microphone: None,
                speaker: None,
            },

            discard_hover_state: false,
            last_discarded_clip: None,

            is_playing: false,
            is_recording: false,
            is_mute: false,
        };

        camera.init_devices();

        camera
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let has_clips = self.clips.len() > 0;

        let on_device_click = ctx
            .link()
            .callback(|_event: MouseEvent| Msg::DisplaySettings());

        match self.view {
            CameraView::Editor => {
                let on_discard_mouseover = ctx
                    .link()
                    .callback(move |_event: MouseEvent| Msg::DiscardHoverState(true));

                let on_discard_mouseout = ctx
                    .link()
                    .callback(move |_event: MouseEvent| Msg::DiscardHoverState(false));

                let on_discard_click = ctx
                    .link()
                    .callback(move |_event: MouseEvent| Msg::OnDiscardClick());

                let on_undo_click = ctx
                    .link()
                    .callback(move |_event: MouseEvent| Msg::OnClipUndo());

                let on_play_click = ctx
                    .link()
                    .callback(move |_event: MouseEvent| Msg::PlaySavedClips());

                let onclick_start = ctx
                    .link()
                    .callback(|_event: MouseEvent| Msg::StartRecording());

                let onclick = ctx
                    .link()
                    .callback(|_event: MouseEvent| Msg::StopRecording());

                let current_timestamp = self.get_current_timestamp() / ctx.props().duration * 100.0;

                let mut hide_undo_button: String = String::from(
                    "absolute right-[-30px] cursor-pointer material-symbols-outlined ",
                );

                if let None = self.last_discarded_clip {
                    hide_undo_button.push_str("hidden");
                }

                let mut show_discard_button:String = String::from("bg-white text-purple py-[2px] px-[8px] rounded-full border-[2px] border-white hover:bg-transparent hover:text-white duration-200 ");

                if self.clips.len() == 0 {
                    hide_undo_button.push_str(" !right-[-10px]");
                    show_discard_button.push_str("hidden");
                }

                let mut show_controls:String = String::from("controls text-white h-[100px] w-full absolute bottom-[15px] z-10 flex flex-row flex-wrap justify-center items-center hidden");

                if self.video_ouput_error.valid {
                    show_controls = show_controls.replace("hidden", "");
                } else {
                    if !show_controls.contains("hidden") {
                        show_controls.push_str("hidden")
                    }
                }

                //Editor View
                html! {
                    <>
                    <div class="video-editor">
                        <div id={self.camera_id.clone()} class="video-wrapper absolute top-0 bottom-0 w-full">
                            <div class="device-error"
                                style={if !self.video_ouput_error.valid {"display:flex;"}else{"display:none"}}>
                                <span class="material-symbols-outlined text-6xl">
                                    {"error"}
                                </span>
                                <p>{"something went wrong! check your device settings."}</p>
                            </div>

                            <Loading load={self.video_ouput_error.valid} />
                        </div>

                        <div class={show_controls}>

                            <div class="flex flex-row justify-center absolute top-[-15px]  w-full center text-xs">
                                <div class="relative"
                                style={if self.is_recording {"display:none;"}else{"display:flex;"}}>
                                    <button
                                    onclick={on_discard_click}
                                    onmouseover={on_discard_mouseover}
                                    onmouseout={on_discard_mouseout}
                                    class={show_discard_button}>
                                        {"discard last clip"}
                                    </button>
                                    <span onclick={on_undo_click} class={hide_undo_button}>
                                        {"undo"}
                                    </span>
                                </div>
                            </div>

                            <span style={ if self.is_recording && self.get_current_timestamp() < self.duration
                                { "display:block;"}else{"display:none;"}}
                                    class="material-symbols-outlined text-[50px] pl-[50px] pr-[10px] text-brand-yellow cursor-pointer hover:opacity-75 duration-200" {onclick}>
                                {"pause"}
                            </span>

                            <span style={if !self.is_recording && self.get_current_timestamp() < self.duration { "display:block;"}else{"display:none;"}}
                                    class="material-symbols-outlined text-[60px] pl-[50px] text-brand-red cursor-pointer hover:opacity-75 duration-200"
                                  onclick={onclick_start}>
                                {"fiber_manual_record"}
                            </span>

                            <span style={if has_clips { "" } else{"opacity:0;cursor:default;"}}
                            onclick={on_play_click}
                            class="material-symbols-outlined text-[50px] cursor-pointer hover:opacity-75 duration-200">
                                {"play_arrow"}
                            </span>

                        </div>
                             <div class="progress-bar bg-purple h-[15px] absolute bottom-0 flex flex-row overflow-hidden w-full">
                                <div class="current-timestamp" style={self.set_percent_style(current_timestamp)}></div>
                                {
                                    self.clips.iter().map(|clip|{

                                        let percentage_from_total = clip.duration / ctx.props().duration * 100.0;

                                        html!{
                                            <div class={classes!({self.set_last_clip_pulse_animation()}, "segments")}
                                             style={self.set_percent_style(percentage_from_total)}></div>
                                        }

                                    }).collect::<Html>()
                                }
                            </div>
                        </div>
                        <div class="actions flex flex-row justify-between items-center pt-5">
                                <button onclick={&on_device_click} class="text-purple flex flex-row items-center justify-center">
                                        <span class="material-symbols-outlined top-[2px] relative">
                                        {"settings"}
                                        </span>
                                        <p class="flex flex-col text-left text-sm pl-[5px] leading-3 text-[12px]">
                                            <span class="text-[8px]">{"Having issues? Maybe adjust your"}</span>
                                            {"Device Settings"}
                                        </p>
                                </button>
                                <button  class="button">{"send feedback"}</button>
                        </div>
                    </>
                }
            }
            CameraView::Preview => {
                let on_exit_preview = ctx
                    .link()
                    .callback(|_event: MouseEvent| Msg::SetView(CameraView::Editor));

                let on_play_toggle = ctx
                    .link()
                    .callback(|_event: MouseEvent| Msg::PreviewOnPlayToggle());

                let on_mute_toggle = ctx
                    .link()
                    .callback(|_event: MouseEvent| Msg::PreviewOnMuteToggle());

                let current_timestamp = self.preview_timestamp / self.preview_duration * 100.0;

                html! {
                    <>
                    <div class="video-preview relative">
                        <button onclick={&on_exit_preview}
                                class="button hover:border-white hover:text-white absolute z-10 top-[15px] right-[15px]">
                            {"exit preview"}
                        </button>
                        <div id={format!("{}-preview", self.camera_id.clone())}
                             class="video-wrapper absolute top-0 bottom-0 "></div>
                        <div class="actions w-full px-[20px] h-[100px]
                            absolute bottom-[15px] text-white text-[50px] flex flex-row justify-between items-center">
                            <span onclick={&on_play_toggle} style={if !self.is_playing {"display:block;"}else{"display:none;"}}
                                  class="material-symbols-outlined text-6xl w-full pl-[30px] text-center
                                      cursor-pointer hover:opacity-75 duration-200">
                                {"play_arrow"}
                            </span>
                            <span  onclick={&on_play_toggle} style={if self.is_playing {"display:block;"}else{"display:none;"}}
                                  class="material-symbols-outlined text-6xl text-brand-yellow w-full pl-[30px] text-center
                                      cursor-pointer hover:opacity-75 duration-200">
                                {"pause"}
                            </span>
                            <span  onclick={&on_mute_toggle} style={if !self.is_mute {"display:block;"}else{"display:none;"}}
                                  class="material-symbols-outlined text-3xl cursor-pointer hover:opacity-75 duration-200">
                                {"volume_up"}
                            </span>
                            <span  onclick={&on_mute_toggle} style={if self.is_mute {"display:block;"}else{"display:none;"}}
                                  class="material-symbols-outlined text-3xl cursor-pointer hover:opacity-75 duration-200">
                                {"volume_off"}
                            </span>
                        </div>

                        <div class="progress-bar bg-purple h-[15px] absolute bottom-0 flex flex-row overflow-hidden w-full">
                            <div class="current-timestamp duration-200 segments" style={self.set_percent_style(current_timestamp)}></div>
                        </div>

                    </div>
                     <div class="actions flex flex-row justify-between items-center pt-5">
                        <button onclick={&on_device_click} class="text-purple flex flex-row items-center justify-center">
                            <span class="material-symbols-outlined top-[2px] relative">
                            {"settings"}
                            </span>
                            <p class="flex flex-col text-left text-sm pl-[5px] leading-3 text-[12px]">
                                <span class="text-[8px]">{"Having issues? Maybe adjust your"}</span>
                                {"Device Settings"}
                            </p>
                        </button>
                        <button  class="button">{"send feedback"}</button>
                    </div>
                    </>
                }
            }
            CameraView::Settings => {
                let on_camera_select = ctx
                    .link()
                    .callback(|e| Msg::SetDevice(e, DeviceType::Camera));

                let on_microphone_select = ctx
                    .link()
                    .callback(|e| Msg::SetDevice(e, DeviceType::Microphone));

                let _on_speaker_select = ctx
                    .link()
                    .callback(|e| Msg::SetDevice(e, DeviceType::Microphone));

                let on_camera_preview_toggle = ctx
                    .link()
                    .callback(move |_event: MouseEvent| Msg::ToggleCameraPreview());

                let on_go_back_click = ctx
                    .link()
                    .callback(|_event: MouseEvent| Msg::DisplayEditor());

                html! {
                    <>
                     <div class="widget-title">
                        <span class="material-symbols-outlined">
                            {"settings"}
                        </span>
                        <div class="w-[80%]">
                            <h3>{"Device Settings"}</h3>
                            <p>{"Adjust your video and audio settings here. "}</p>
                        </div>
                    </div>
                    <h3>{"Video"}</h3>
                    <div class="relative">
                        <div id={self.camera_id.clone()} class={classes!({"camera-preview"}, self.is_camera_preview_active())}></div>
                    </div>
                    <div onclick={&on_camera_preview_toggle} class={classes!({"camera-preview-overlay"},
                                                                             self.is_camera_preview_active())}></div>
                    <div class="dropdown">
                        <div class={classes!("wrapper","!w-[90%]",{self.does_device_error_exist(DeviceType::Camera)})}>
                            <p>{"Camera"}</p>
                            <select name="camera" id="camera" onchange={on_camera_select}>
                                {
                                    self.devices.iter()
                                    .filter(|m| m.kind() == MediaDeviceKind::Videoinput).map(|media_info|{
                                        html!{
                                            <option value={media_info.device_id()}
                                            selected={
                                                match self.active_devices.clone().camera {
                                                    Some(id)=> id == media_info.device_id(),
                                                    None => false
                                                }
                                            }
                                            >{media_info.label()}</option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                        </div>
                        <span class="material-symbols-outlined cursor-pointer hover:text-purple duration-200 relative z-50"
                              onclick={on_camera_preview_toggle}>
                            {"play_arrow"}
                        </span>
                        <div class={classes!("w-full","text-[12px]","p-[9px]",{self.does_device_error_exist(DeviceType::Camera)})}>
                                {self.video_ouput_error.message.clone()}
                        </div>
                    </div>

                    <h3 class="mt-2">{"Audio"}</h3>
                    <div class="dropdown">
                        <div class="wrapper mb-[10px]">
                            <p class="w-[30%]">{"Microphone"}</p>
                            <select name="microphone" id="microphone" onchange={on_microphone_select}>
                                {
                                    self.devices.iter()
                                    .filter(|m| m.kind() == MediaDeviceKind::Audioinput).map(|media_info|{
                                        html!{
                                            <option value={media_info.device_id()}
                                                selected={
                                                    match self.active_devices.clone().camera {
                                                        Some(id)=> id == media_info.device_id(),
                                                        None => false
                                                    }
                                                }
                                                >{media_info.label()}</option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                        </div>
                        <div class="w-full flex flex-row justify-between items-center mb-5 pr-[8px]">
                            <span class="material-symbols-outlined relative  text-[25px] text-purple">
                                {"keyboard_voice"}
                            </span>
                            <div class="mic-level-wrapper w-[90%] h-[10px] bg-light-gray flex flex-row rounded-full overflow-hidden">
                                <div class="mic-level bg-purple duration-200" style={self.get_microphone_state_styles()}></div>
                            </div>
                        </div>
                    </div>
                    <button  onclick={on_go_back_click} class="button">{"go back"}</button>

                    </>
                }
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link().clone();

        match msg {
            Msg::SetStreamRecorder(stream, recorder) => {
                self.stream = stream;
                self.recorder = recorder;
                return true;
            }
            Msg::CameraServiceMessenger(context) => match context.context_type.unwrap() {
                CameraContextAction::SendBlobChunk => {
                    self.chunks
                        .push(&JsValue::from(context.chunk.clone().unwrap()));
                }
                CameraContextAction::SendDeviceList => {
                    self.devices = context.devices.unwrap();
                    return true;
                }
                CameraContextAction::SendUserMedia => {
                    //Reset device errors everytime user media get reloaded
                    self.video_ouput_error = DeviceError {
                        device_type: DeviceType::Camera,
                        message: "".to_string(),
                        valid: true,
                    };

                    self.audio_input_error = DeviceError {
                        device_type: DeviceType::Microphone,
                        message: "".to_string(),
                        valid: true,
                    };

                    self.stream = context.stream.unwrap();

                    let mut recorder_optiona = MediaRecorderOptions::new();
                    recorder_optiona.mime_type("video/webm");

                    self.recorder =
                        MediaRecorder::new_with_media_stream_and_media_recorder_options(
                            &self.stream,
                            &recorder_optiona,
                        )
                        .unwrap();

                    let document = window().unwrap().document().unwrap();

                    let video_ele: HtmlMediaElement = document
                        .create_element("video")
                        .unwrap()
                        .dyn_into::<HtmlMediaElement>()
                        .unwrap();

                    let mut vid_id = String::from("vid-");
                    vid_id.push_str(self.camera_id.as_str());

                    video_ele.set_attribute("id", &vid_id).unwrap_throw();

                    video_ele.set_src_object(Some(&self.stream));
                    video_ele.set_muted(true);

                    let video_wrapper =
                        document.get_element_by_id(self.camera_id.as_str()).unwrap();
                    video_wrapper.set_inner_html("");
                    video_wrapper.append_child(&video_ele).unwrap();

                    let _ = video_ele.play();

                    self.audio_analyser.set_fft_size(512);
                    self.audio_analyser.set_min_decibels(-127.0);
                    self.audio_analyser.set_max_decibels(0.0);
                    self.audio_analyser.set_smoothing_time_constant(0.4);

                    let microphone = &self
                        .audio_context
                        .create_media_stream_source(&self.stream)
                        .unwrap();

                    let audo_node = self.audio_analyser.clone();

                    microphone.connect_with_audio_node(&audo_node).unwrap();

                    self.audio_interval_timer = Interval::new(100, move || {
                        CameraService::dispatcher().send(Request::SendMicFrequency());
                    });

                    return true;
                }
                CameraContextAction::SendMicFrequency => {
                    let volume = &self.audio_analyser.frequency_bin_count();
                    let mut volumes = volume.to_le_bytes();

                    let mut last_volume: u64 = 0;

                    let _ = &self.audio_analyser.get_byte_frequency_data(&mut volumes);

                    for i in volumes.iter() {
                        let x = u64::try_from(*i).expect("expected_u64_cast");
                        last_volume += x;
                    }

                    let length = volumes.len();
                    let avg_volume = last_volume / u64::try_from(length).unwrap();

                    if avg_volume > 60 {
                        self.audio_average_percent = avg_volume - 60;
                    }

                    return true;
                }
                CameraContextAction::SendDeviceError => {
                    let error = context.device_error.unwrap();
                    match error.device_type {
                        DeviceType::Camera => {
                            self.video_ouput_error = error;
                        }
                        DeviceType::Microphone => {
                            self.audio_input_error = error;
                        }
                        DeviceType::Speaker => todo!(),
                    }
                    return true;
                }
                CameraContextAction::SendClipDetails => {
                    self.clips.push(context.clip_details.unwrap());
                    self.timestamp = 0.0;
                    return true;
                }
                CameraContextAction::AddedClip => {
                    let clip_details = context.clip_details.unwrap();
                    let last_chunk = self
                        .chunks
                        .get(self.chunks.length() - 1)
                        .dyn_into::<JsValue>()
                        .unwrap()
                        .dyn_into::<Blob>()
                        .unwrap();

                    self.get_clip_detail_from_blob(last_chunk, clip_details.id.clone());
                }
                CameraContextAction::MergedClip => {
                    self.play_saved_clip(ctx);
                }
            },
            Msg::Timestamp() => {
                self.timestamp += 0.1;

                let _timeleft = self.get_time_left();

                self.current_timestamp = self.get_current_timestamp();

                if self.current_timestamp >= self.duration {
                    self.timestamp_timer = Interval::new(1000, || {});
                    self.stop_recorder();
                    self.is_recording = false;
                }
            }
            Msg::StartRecording() => {
                self.start_recorder(ctx);

                self.is_recording = true;

                self.last_discarded_clip = None;

                self.timestamp_timer = Interval::new(100, move || {
                    link.clone().send_message(Msg::Timestamp());
                });

                return true;
            }
            Msg::StopRecording() => {
                self.timestamp_timer = Interval::new(1000, || {});

                self.is_recording = false;

                self.stop_recorder();
            }
            Msg::SetView(view) => {
                self.preview_timestamp_timer = Interval::new(1000, || {});

                self.preview_timestamp = 0.0;

                //BUG: VNode not being destroyed after hydration.
                let doc = window().unwrap().document().unwrap();

                let preview_video_element = doc.get_element_by_id(
                    format!("vid-{}-preview-player", self.camera_id.clone()).as_str(),
                );

                if let Some(ele) = preview_video_element {
                    ele.remove();
                }

                match view {
                    CameraView::Editor => {
                        self.init_devices();
                    }
                    _ => (),
                }

                self.view = view;

                return true;
            }
            Msg::SetDeviceID(id, device_type) => {
                match device_type {
                    DeviceType::Camera => self.active_devices.camera = Some(id),
                    DeviceType::Microphone => self.active_devices.microphone = Some(id),
                    DeviceType::Speaker => self.active_devices.speaker = Some(id),
                }
                self.init_devices();
            }
            Msg::SetDevice(event, device_type) => {
                let link = ctx.link();

                match device_type {
                    DeviceType::Camera => {
                        let cb = link.callback(move |field_value: FieldValue| {
                            Msg::SetDeviceID(field_value.value, device_type.clone())
                        });

                        FormBuilder::<ActiveDevices>::convert_event_and_set(event, cb);
                    }
                    DeviceType::Microphone => {
                        let cb = link.callback(move |field_value: FieldValue| {
                            Msg::SetDeviceID(field_value.value, device_type.clone())
                        });

                        FormBuilder::<ActiveDevices>::convert_event_and_set(event, cb);
                    }
                    DeviceType::Speaker => {
                        let cb = link.callback(move |field_value: FieldValue| {
                            Msg::SetDeviceID(field_value.value, device_type.clone())
                        });

                        FormBuilder::<ActiveDevices>::convert_event_and_set(event, cb);
                    }
                }
            }
            Msg::ToggleCameraPreview() => {
                self.is_camera_preview_active = !self.is_camera_preview_active;
            }
            Msg::DisplaySettings() => {
                self.view = CameraView::Settings;
                self.init_devices();
                return true;
            }
            Msg::DisplayEditor() => {
                self.view = CameraView::Editor;
                self.init_devices();
                return true;
            }
            Msg::DiscardHoverState(state) => {
                self.discard_hover_state = state;
                return true;
            }
            Msg::OnDiscardClick() => {
                self.discard_last_clip();
                return true;
            }
            Msg::OnClipUndo() => {
                if let Some(clip) = &self.last_discarded_clip {
                    self.clips.push(clip.clone());
                    self.last_discarded_clip = None;
                    self.timestamp = 0.0;
                }
                return true;
            }
            Msg::PlaySavedClips() => {
                if !self.is_recording {
                    self.is_playing = true;
                    self.view = CameraView::Preview;

                    self.merge_stored_clips();
                }
            }
            Msg::PreviewOnPlayToggle() => {
                self.is_playing = !self.is_playing;

                let doc = window().unwrap().document().unwrap();

                let preview_video_element = doc.get_element_by_id(
                    format!("vid-{}-preview-player", self.camera_id.clone()).as_str(),
                );

                if let Some(element) = preview_video_element {
                    let ele = element.dyn_into::<HtmlMediaElement>().unwrap();

                    if self.is_playing {
                        let _ = ele.play().unwrap();
                        let link = ctx.link().clone();
                        self.preview_timestamp_timer = Interval::new(100, move || {
                            link.clone().send_message(Msg::PreviewTimestamp());
                        });
                    } else {
                        let _ = ele.pause().unwrap();

                        self.preview_timestamp_timer = Interval::new(1000, || {});
                    }
                }

                return true;
            }
            Msg::PreviewOnMuteToggle() => {
                self.is_mute = !self.is_mute;

                let doc = window().unwrap().document().unwrap();

                let preview_video_element = doc.get_element_by_id(
                    format!("vid-{}-preview-player", self.camera_id.clone()).as_str(),
                );

                if let Some(element) = preview_video_element {
                    let ele = element.dyn_into::<HtmlMediaElement>().unwrap();

                    ele.set_muted(self.is_mute);
                }

                return true;
            }
            Msg::PreviewTimestamp() => {
                //TODO:: Ensure video data is loaded. Preview duration may not be accessable.
                let doc = window().unwrap().document().unwrap();

                let preview_video_element = doc.get_element_by_id(
                    format!("vid-{}-preview-player", self.camera_id.clone()).as_str(),
                );

                if let Some(element) = preview_video_element {
                    let ele = element.dyn_into::<HtmlMediaElement>().unwrap();

                    ele.set_muted(self.is_mute);

                    self.preview_timestamp = ele.current_time();
                    self.preview_duration = ele.duration();

                    if self.preview_timestamp >= self.preview_duration {
                        self.preview_timestamp_timer = Interval::new(1000, || {});
                        self.preview_timestamp = 0.0;
                        self.is_playing = false;
                    }
                }

                return true;
            }
        }
        false
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}

    fn destroy(&mut self, _ctx: &Context<Self>) {}
}

impl Camera {
    pub fn set_last_clip_pulse_animation(&self) -> String {
        let mut classes = String::from("");
        if self.discard_hover_state {
            classes.push_str(" last:animate-pulse");
        }
        classes
    }
    pub fn get_current_timestamp(&self) -> f64 {
        self.duration - self.get_time_left()
    }

    pub fn get_time_left(&self) -> f64 {
        let mut clips_total_duration: f64 = 0.0;

        for clip in self.clips.iter() {
            clips_total_duration += clip.duration;
        }

        self.duration - (self.timestamp + clips_total_duration)
    }

    pub fn get_microphone_state_styles(&self) -> String {
        format!("width:{}%;", self.audio_average_percent)
    }
    pub fn set_percent_style(&self, percent: f64) -> String {
        format!("width:{}%;", percent)
    }

    pub fn is_camera_preview_active(&self) -> String {
        if self.is_camera_preview_active {
            String::from(" active")
        } else {
            String::from("")
        }
    }

    pub fn does_device_error_exist(&self, device_type: DeviceType) -> String {
        match device_type {
            DeviceType::Camera => {
                if !self.video_ouput_error.valid {
                    String::from("border-brand-red has_error text-brand-red")
                } else {
                    String::from("")
                }
            }
            DeviceType::Microphone => {
                if !self.audio_input_error.valid {
                    String::from("has_error")
                } else {
                    String::from("")
                }
            }
            DeviceType::Speaker => todo!(),
        }
    }

    pub fn init_devices(&mut self) {
        let window = web_sys::window().expect("Missing Window");

        let media = window.navigator().media_devices().unwrap();

        let mut constraints = MediaStreamConstraints::new();

        match &self.active_devices.camera {
            Some(id) => {
                let contstraint_options = ConstraintOptions {
                    device_id: ConstraintDeviceId { exact: id.clone() },
                };
                constraints.video(
                    &JsValue::from_serde(&contstraint_options).expect("contstraint_options_error"),
                );
            }

            None => {
                constraints.video(&JsValue::from(true));
            }
        }

        match &self.active_devices.microphone {
            Some(id) => {
                let contstraint_options = ConstraintOptions {
                    device_id: ConstraintDeviceId { exact: id.clone() },
                };
                constraints.audio(
                    &JsValue::from_serde(&contstraint_options).expect("contstraint_options_error"),
                );
            }
            None => {
                constraints.audio(&JsValue::from(true));
            }
        }

        let user_media = media.get_user_media_with_constraints(&constraints).unwrap();

        self.audio_context = AudioContext::new().unwrap();
        self.audio_analyser = self.audio_context.create_analyser().unwrap();

        spawn_local(async move {
            let mut device_list = vec![];

            let devices = JsFuture::from(media.enumerate_devices().unwrap_throw());
            let device_response = devices.await;

            match device_response {
                Ok(res) => {
                    let _devices = Array::from(&res);

                    for device in _devices.iter() {
                        let device_info = MediaDeviceInfo::from(device);
                        device_list.push(device_info);
                    }
                    CameraService::dispatcher().send(Request::SendDeviceList(device_list));
                }
                Err(err) => {
                    log::info!("{:?}", err);
                }
            }
        });

        spawn_local(async move {
            let _user_media = JsFuture::from(user_media).await;

            match _user_media {
                Ok(media) => {
                    let stream: MediaStream = media.try_into().unwrap();
                    CameraService::dispatcher().send(Request::SendUserMedia(stream));
                }
                Err(err) => {
                    CameraService::dispatcher().send(Request::SendDeviceError(err));
                }
            }
        });
    }

    pub fn start_recorder(&mut self, _ctx: &Context<Self>) {
        let ondata_callback = Closure::wrap(Box::new(move |e: BlobEvent| {
            let data = e.data().expect("expect_data");
            let _size = data.size();

            CameraService::dispatcher().send(Request::SendBlobChunk(data))
        }) as Box<dyn FnMut(BlobEvent)>);

        self.recorder
            .set_ondataavailable(Some(ondata_callback.as_ref().unchecked_ref()));

        ondata_callback.forget();

        self.recorder.start().unwrap();
    }

    pub fn get_clip_detail_from_blob(&self, chunk: Blob, id: String) {
        let doc = window().unwrap().document().unwrap();

        let chunk_array = Array::new();
        chunk_array.push(&chunk);

        let mut options = BlobPropertyBag::new();
        options.type_("video/webm");

        let blob = Blob::new_with_blob_sequence_and_options(&chunk_array, &options).unwrap();
        let blob_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

        let vid_ele = doc
            .create_element("video")
            .unwrap()
            .dyn_into::<HtmlMediaElement>()
            .unwrap();

        vid_ele.set_src(blob_url.as_str());
        vid_ele.load();

        let onloadedmetadata_callback = Closure::wrap(Box::new(move |e: Event| {
            let video_element_target = e.target().unwrap().dyn_into::<HtmlVideoElement>().unwrap();

            let _chunk = chunk.clone();
            let _id = id.clone();

            if video_element_target.duration() == f64::INFINITY {
                video_element_target.set_current_time(f64::MAX);

                let ontimeupdate_cb = Closure::wrap(Box::new(move |_e: Event| {
                    let __chunk = _chunk.clone();
                    let __id = _id.clone();

                    let temp_element_target =
                        _e.target().unwrap().dyn_into::<HtmlVideoElement>().unwrap();

                    temp_element_target.set_ontimeupdate(None);
                    temp_element_target.set_current_time(f64::from(0));
                    temp_element_target.set_onloadedmetadata(None);

                    let clip_detail = ClipDetails {
                        id: __id,
                        duration: temp_element_target.duration(),
                        chunk: __chunk,
                    };

                    CameraService::dispatcher().send(Request::SendClipDetails(clip_detail));
                }) as Box<dyn FnMut(Event)>);

                video_element_target
                    .set_ontimeupdate(Some(ontimeupdate_cb.as_ref().unchecked_ref()));
                ontimeupdate_cb.forget();
            }
        }) as Box<dyn FnMut(Event)>);

        vid_ele.set_onloadedmetadata(Some(onloadedmetadata_callback.as_ref().unchecked_ref()));

        onloadedmetadata_callback.forget();
    }
    pub fn send_recording(&self) {
        let doc = window().unwrap().document().unwrap();

        let chunks = JsValue::from(&self.chunks);

        let mut options = BlobPropertyBag::new();
        options.type_("video/webm");

        let blob = Blob::new_with_blob_sequence_and_options(&chunks, &options).unwrap();
        let blob_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

        let vid_ele = doc
            .create_element("video")
            .unwrap()
            .dyn_into::<HtmlMediaElement>()
            .unwrap();

        vid_ele.set_current_time(f64::from(0));
        vid_ele.set_src(blob_url.as_str());
        vid_ele.load();

        let a = Array::new();
        a.push(&vid_ele);

        let mut vid_id = String::from("vid-");
        vid_id.push_str(self.camera_id.as_str());

        self.video_element.replace_with_with_node(&a).unwrap();
        self.video_element.load();
    }
    pub fn play_saved_clip(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();

        let doc = window()
            .expect("expect_window")
            .document()
            .expect("expect_document");

        let vid_ele = doc
            .create_element("video")
            .unwrap()
            .dyn_into::<HtmlMediaElement>()
            .unwrap();

        let mut vid_id = String::from("vid-");
        vid_id.push_str(self.camera_id.as_str());

        vid_ele
            .set_attribute("id", format!("{}-preview-player", &vid_id).as_str())
            .unwrap_throw();

        let mut video_uri = Utilities::config("api_url");
        video_uri.push_str(format!("clip/session?v={}", Utilities::rnd_id("cache-")).as_str());
        vid_ele.set_src(&video_uri);
        vid_ele.load();

        self.preview_timestamp_timer = Interval::new(100, move || {
            link.clone().send_message(Msg::PreviewTimestamp());
        });

        let video_wrapper = doc
            .get_element_by_id(format!("{}-preview", self.camera_id).as_str())
            .unwrap();

        video_wrapper.set_inner_html("");
        video_wrapper.append_child(&vid_ele).unwrap();

        let _ = vid_ele.play().unwrap();
    }
    pub fn merge_stored_clips(&self) {
        let clip_ids = self
            .clips
            .iter()
            .map(|c| ClipDetailRequest {
                id: c.id.clone(),
                duration: c.duration.clone(),
            })
            .collect::<Vec<ClipDetailRequest>>();

        CameraService::dispatcher().send(Request::OnPlayback(clip_ids));
    }
    pub fn discard_last_clip(&mut self) {
        if let Some(clip) = self.clips.pop() {
            self.last_discarded_clip = Some(clip);
            self.timestamp = 0.0;
        }
    }
    pub fn stop_recorder(&mut self) {
        let _ = self.recorder.stop().unwrap();
    }
}
