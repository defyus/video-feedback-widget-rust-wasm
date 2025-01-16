use field_accessor::FieldAccessor;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::{Blob, MediaDeviceInfo, MediaRecorder, MediaStream};
use yew::Event;

pub enum Request {
    SendBlobChunk(Blob),
    OnPlayback(Vec<ClipDetailRequest>),
    OnSubmission(),
    SendDeviceList(Vec<MediaDeviceInfo>),
    SendUserMedia(MediaStream),
    SendClipDetails(ClipDetails),
    SendDeviceError(JsValue),
    SendMicFrequency(),
}

#[derive(Clone, Serialize, Debug)]
pub struct DeviceSettingForm {
    pub video: FormField,
    pub audio_input: FormField,
    pub audio_output: FormField,
}

#[derive(Clone, Debug)]
pub struct CameraContext {
    pub context_type: Option<CameraContextAction>,
    pub stream: Option<MediaStream>,
    pub recorder: Option<MediaRecorder>,
    pub chunk: Option<Blob>,
    pub devices: Option<Vec<MediaDeviceInfo>>,
    pub device_error: Option<DeviceError>,
    pub clip_details: Option<ClipDetails>,
    pub merged_clip: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub enum CameraContextAction {
    SendBlobChunk,
    AddedClip,
    SendDeviceList,
    SendUserMedia,
    SendClipDetails,
    SendMicFrequency,
    SendDeviceError,
    MergedClip,
}

impl CameraContextAction {}

#[derive(Deserialize, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub kind: String,
    pub label: String,
    pub group_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipDetailRequest {
    pub id: String,
    pub duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstraintOptions {
    pub device_id: ConstraintDeviceId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstraintDeviceId {
    pub exact: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveDevices {
    pub camera: Option<String>,
    pub microphone: Option<String>,
    pub speaker: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceError {
    pub device_type: DeviceType,
    pub message: String,
    pub valid: bool,
}

#[derive(Debug, Clone)]
pub struct ClipDetails {
    pub id: String,
    pub duration: f64,
    pub chunk: Blob,
}

#[derive(PartialEq, Clone, Debug)]
pub enum DeviceType {
    Camera,
    Microphone,
    Speaker,
}

#[derive(PartialEq, Clone)]
pub enum CameraView {
    Editor,
    Preview,
    Settings,
}

#[derive(PartialEq, Clone)]
pub enum FeedbackStep {
    TypeSelection,
    Message,
    Video,
    DeviceSettings,
    VideoEditor,
    ThankYou,
    None,
    GoBack,
}

pub enum Msg {
    SetStep(FeedbackStep),
    UpdateVideoFormFieldValue(Event),
    SetVideoFormFieldValue(FieldValue),
    UpdateMessageFormFieldValue(Event),
    SetMessageFormFieldValue(FieldValue),
    Toggle(),
    Close(),
    FeedbackService(String),
}

#[derive(Clone, Serialize, Debug)]
pub struct FeedbackMsg {
    pub name: FormField,
    pub email: FormField,
    pub your_message: FormField,
}

#[derive(Clone, Serialize, Debug)]
pub struct FeedbackVideo {
    pub name: FormField,
    pub email: FormField,
}

#[derive(Default, Clone, FieldAccessor, Deserialize, Serialize, Debug)]
pub struct FormField {
    pub label: &'static str,
    pub value: &'static str,
    pub required: &'static str,
    pub icon: &'static str,
    pub field_type: &'static str,
    pub validator: &'static str,
    pub sort_order: &'static str,
    pub options: Vec<FormSelectOption>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct FormSelectOption {
    value: &'static str,
    label: &'static str,
    selected: &'static str,
}
pub enum FormFieldType {
    Text,
    TextArea,
    Email,
    Hidden,
}

pub struct FieldValue {
    pub id: String,
    pub value: String,
}
