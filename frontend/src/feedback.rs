use std::collections::HashMap;

use crate::camera::Camera;
use crate::form::FormBuilder;
use crate::models::{
    CameraView, FeedbackMsg, FeedbackStep, FeedbackVideo, FieldValue, FormField, Msg,
};

use crate::service::feedback::{FeedbackService, Request};
use crate::utilities::Utilities;

use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDocument};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

pub struct FeedbackWidget {
    widget_id: String,
    active_step: FeedbackStep,
    previous_step: FeedbackStep,
    active: bool,
    video_form: FormBuilder<FeedbackVideo>,
    message_form: FormBuilder<FeedbackMsg>,
    _fs: Dispatcher<FeedbackService>,
    producer: Box<dyn Bridge<FeedbackService>>,
}

impl FeedbackWidget {
    fn is_step_active(&self, step: FeedbackStep) -> String {
        let mut classes = String::from("step ");
        if self.active_step == step {
            classes.push_str(" active");
        }
        classes
    }

    fn is_widget_active(&self, prefix: &str) -> String {
        let mut classes = String::from(prefix);
        if self.active {
            classes.push_str(" active");
        }
        classes
    }

    fn set_video_form_field(&mut self, field_value: FieldValue) {
        let value = Utilities::string_to_static_str(field_value.value.clone());
        let field = self
            .video_form
            .keys
            .get_mut(field_value.id.as_str())
            .unwrap();
        let _value = field.get_mut("value").unwrap();

        *_value = value.to_string();
    }
    fn set_message_form_field(&mut self, field_value: FieldValue) {
        let value = Utilities::string_to_static_str(field_value.value.clone());
        let field = self
            .message_form
            .keys
            .get_mut(field_value.id.as_str())
            .unwrap();
        let _value = field.get_mut("value").unwrap();

        *_value = value.to_string();
    }
}
impl Component for FeedbackWidget {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let html_document = document.dyn_into::<HtmlDocument>().unwrap();

        let mut cookie = Utilities::rnd_id("X-FDot-Session=");
        cookie.push_str("; path=/");
        let mut _cookie = Utilities::rnd_id("teseter=");
        _cookie.push_str("; path=/");

        html_document.set_cookie(_cookie.as_str()).unwrap();
        html_document.set_cookie(cookie.as_str()).unwrap();

        let link = ctx.link();

        let rnd_id = Utilities::rnd_id("");
        let mut widget_id = String::from("feedback-widget-");
        widget_id.push_str(rnd_id.as_str());

        let feedback_msg = FeedbackMsg {
            name: FormField {
                label: "Name",
                validator: "",
                field_type: "text",
                icon: "badge",
                sort_order: "0",
                value: "",
                required: "true",
                options: vec![],
            },
            email: FormField {
                label: "Email Address",
                validator: "",
                field_type: "email",
                icon: "alternate_email",
                sort_order: "1",
                value: "",
                required: "true",
                options: vec![],
            },
            your_message: FormField {
                label: "Your message",
                validator: "",
                field_type: "textarea",
                icon: "mail",
                sort_order: "2",
                value: "",
                required: "true",
                options: vec![],
            },
        };

        let mut feedback_msg_form = FormBuilder::new(
            Utilities::rnd_id("feedback-"),
            feedback_msg,
            String::from("send feedback"),
        );

        let msg_fields = feedback_msg_form.field_keys.clone();

        feedback_msg_form.on_submit = link.callback(move |event: FocusEvent| {
            event.prevent_default();

            let data = Utilities::form_data(event, msg_fields.clone());
            log::info!("{:?}", &data);
            FeedbackService::dispatcher().send(Request::OnMessageSubmission(
                serde_json::to_string::<HashMap<String, String>>(&data).unwrap(),
            ));
            Msg::SetStep(FeedbackStep::ThankYou)
        });

        let feedback_vid = FeedbackVideo {
            name: FormField {
                label: "Name",
                validator: "",
                field_type: "text",
                icon: "badge",
                sort_order: "0",
                value: "",
                required: "true",
                options: vec![],
            },
            email: FormField {
                label: "Email Address",
                validator: "",
                field_type: "email",
                icon: "alternate_email",
                sort_order: "1",
                value: "",
                required: "true",
                options: vec![],
            },
        };

        let mut feedback_video_form = FormBuilder::new(
            Utilities::rnd_id("feedback-"),
            feedback_vid,
            String::from("start recording"),
        );

        let video_fields = feedback_video_form.field_keys.clone();

        feedback_video_form.on_submit = link.callback(move |event: FocusEvent| {
            event.prevent_default();

            let _data = Utilities::form_data(event, video_fields.clone());

            Msg::SetStep(FeedbackStep::VideoEditor)
        });

        feedback_video_form.on_change =
            link.callback(move |event: Event| Msg::UpdateVideoFormFieldValue(event));

        Self {
            widget_id,
            active_step: FeedbackStep::TypeSelection,
            previous_step: FeedbackStep::None,
            active: false,
            video_form: feedback_video_form,
            message_form: feedback_msg_form,
            _fs: FeedbackService::dispatcher(),
            producer: FeedbackService::bridge(ctx.link().callback(Msg::FeedbackService)),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let on_toggle_click = link.callback(|_event: MouseEvent| Msg::Toggle());

        let on_msg_selection =
            link.callback(|_event: MouseEvent| Msg::SetStep(FeedbackStep::Message));

        let on_vid_selection =
            link.callback(|_event: MouseEvent| Msg::SetStep(FeedbackStep::Video));

        let _on_startover_click =
            link.callback(|_event: MouseEvent| Msg::SetStep(FeedbackStep::TypeSelection));

        let on_close_click = link.callback(|_event: MouseEvent| Msg::Close());

        let _on_go_back_click =
            link.callback(|_event: MouseEvent| Msg::SetStep(FeedbackStep::GoBack));

        let _on_device_click =
            link.callback(move |_event: MouseEvent| Msg::SetStep(FeedbackStep::DeviceSettings));

        let msg_form_html = self.message_form.html();
        let video_form_html = self.video_form.html();

        html! {
            <>
                <div id={self.widget_id.clone()} class={classes!({self.is_widget_active("feedback-widget")})}>
                    <div class={classes!({self.is_widget_active("pane")})}>
                        <div class="title ">
                            <h3>{"Leave some "}<b>{"Feedback"}</b></h3>
                            <p class="text-[11px] text-light-purple ">
                                {"Interested in sharing your experience. Have any suggestions or
                                    recommendations to help improve visitors experience?"}
                            </p>
                        </div>
                        <div class="bg-white p-5  rounded-bl-lg  rounded-br-lg duration-500">
                            <div class={classes!({self.is_step_active(FeedbackStep::TypeSelection)})}>
                                <div class="type-selector" onclick={on_msg_selection}>
                                    <span class="material-symbols-outlined text-white text-[40px]">
                                        {"forum"}
                                    </span>
                                    <div class="w-[80%]">
                                        <h3 class="text-white"><b>{"Feedback"}</b></h3>
                                        <p class="text-[12px] text-light-purple ">{"Interested in sharing your experience. "}</p>
                                    </div>
                                </div>
                                <div class="type-selector" onclick={on_vid_selection}>
                                    <span class="material-symbols-outlined text-white text-[40px]">
                                        {"forum"}
                                    </span>
                                    <div class="w-[80%]">
                                        <h3 class="text-white"><b>{"Video Feedback"}</b></h3>
                                        <p class="text-[12px] text-light-purple ">{"Interested in sharing your experience. "}</p>
                                    </div>
                                </div>
                            </div>
                            <div class={classes!({self.is_step_active(FeedbackStep::Message)})}>
                                <div class="flex flex-wrap justify-between items-center mb-[15px]">
                                    <span class="material-symbols-outlined text-[60px]">
                                        {"forum"}
                                    </span>
                                    <div class="w-[80%]">
                                        <h3 ><b>{"Feedback"}</b></h3>
                                        <p class="text-[12px] ">{"Interested in sharing your experience. "}</p>
                                    </div>
                                </div>
                                {msg_form_html}
                            </div>
                            <div class={classes!({self.is_step_active(FeedbackStep::Video)})}>
                                <div class="flex flex-wrap justify-between items-center mb-[15px]">
                                    <span class="material-symbols-outlined text-[60px]">
                                        {"forum"}
                                    </span>
                                    <div class="w-[80%]">
                                        <h3 ><b>{"Video Feedback"}</b></h3>
                                        <p class="text-[12px] ">{"Interested in sharing your experience. "}</p>
                                    </div>
                                </div>
                                {video_form_html}
                            </div>
                            <div class={classes!({self.is_step_active(FeedbackStep::VideoEditor)})}>
                                <Camera duration={f64::from(10)} view={CameraView::Editor}/>
                            </div>
                            <div class={classes!({self.is_step_active(FeedbackStep::ThankYou)})}>
                                <div class="w-full text-center">
                                    <h3 ><b>{"Thank you!"}</b></h3>
                                    <p class="text-[12px] mb-5">{"We have received your thoughtful feedback."}</p>
                                    <button onclick={&on_close_click} class="btn-primary w-full">{"close"}</button>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div onclick={&on_toggle_click}
                         class="toggle rounded-full bg-purple h-[50px] w-[50px] text-white
                                hover:bg-dark-blue-purple shadow-sm hover:shadow-lg duration-200 cursor-pointer center">
                        <span class="material-symbols-outlined text-white text-[20px] relative top-[2px]">
                            {"forum"}
                        </span>
                    </div>
                </div>

            </>

        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetStep(id) => {
                let previous_step = self.active_step.clone();

                if id == FeedbackStep::GoBack {
                    self.active_step = self.previous_step.clone();
                    self.previous_step = previous_step;
                } else {
                    self.active_step = id;
                    self.previous_step = previous_step;
                }

                return true;
            }
            Msg::UpdateVideoFormFieldValue(event) => {
                let link = ctx.link();

                let cb = link.callback(|field_value| Msg::SetVideoFormFieldValue(field_value));

                FormBuilder::<FeedbackVideo>::convert_event_and_set(event, cb);
            }
            Msg::SetVideoFormFieldValue(field_value) => {
                self.set_video_form_field(field_value);
            }
            Msg::UpdateMessageFormFieldValue(event) => {
                let link = ctx.link();

                let cb = link.callback(|field_value| Msg::SetMessageFormFieldValue(field_value));
                FormBuilder::<FeedbackMsg>::convert_event_and_set(event, cb);
            }
            Msg::SetMessageFormFieldValue(field_value) => {
                self.set_message_form_field(field_value);
            }
            Msg::Toggle() => {
                self.active = !self.active;
                self.active_step = FeedbackStep::TypeSelection;
                return true;
            }
            Msg::Close() => {
                self.active = false;
                self.active_step = FeedbackStep::TypeSelection;
                return true;
            }
            Msg::FeedbackService(context) => {
                log::info!("{:?} <-- component", context);
            }
        }
        return false;
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}

    fn destroy(&mut self, _ctx: &Context<Self>) {}
}
