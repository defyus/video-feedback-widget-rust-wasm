use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{
    window, Event, EventTarget, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Node,
};
use yew::prelude::*;
use yew::Callback;

use crate::models::FieldValue;

pub struct FormBuilder<T: Serialize + Clone> {
    pub id: String,
    pub context: T,
    pub on_submit: Callback<FocusEvent>,
    pub on_change: Callback<Event>,
    pub submit_label: String,
    pub(super) field_keys: Vec<String>,
    pub(super) keys: HashMap<String, HashMap<String, String>>,
}

impl<T: Serialize + Clone> FormBuilder<T> {
    pub fn build_form(&mut self) {
        let mut _keys = serde_json::to_string::<T>(&self.context.clone()).unwrap();

        _keys = _keys.replace("[", "\"[");
        _keys = _keys.replace("]", "]\"");

        self.keys =
            serde_json::from_str::<HashMap<String, HashMap<String, _>>>(_keys.as_str()).unwrap();

        for key in self.keys.keys().into_iter() {
            self.field_keys.push(key.clone());
        }
    }

    pub fn html(&self) -> Html {
        let document = window().unwrap().document().unwrap();

        let mut fields_output = vec![];

        let mut _sorted: Vec<_> = self.keys.iter().collect();

        _sorted.sort_by_key(|(_key, value)| value.get("sort_order").unwrap());

        for (key, value) in _sorted.iter() {
            let field_type = value.get("field_type").unwrap();

            let mut input =
                format!(
                "<{} class=\"field\" type=\"{}\" name=\"{}\" placeholder=\"{}\" value=\"{}\" {}",
                if field_type == "textarea"{
                    String::from("textarea")
                }else if field_type == "select"{
                    String::from("select")
                }else{
                    String::from("input")
                },
                value.get("field_type").unwrap(),
                key,
                value.get("label").unwrap(),
                value.get("value").unwrap(),
                if value.get("required").unwrap() == "true" { "required "}else{ "" },
            );

            let input_closure = if field_type == "textarea" {
                format!(
                    " ></textarea> <span class=\"material-symbols-outlined\">{}</span>",
                    value.get("icon").unwrap()
                )
            } else if field_type == "select" {
                format!(
                    " ></select> <span class=\"material-symbols-outlined\">{}</span>",
                    value.get("icon").unwrap()
                )
            } else {
                format!(
                    "/> <span class=\"material-symbols-outlined\">{}</span>",
                    value.get("icon").unwrap()
                )
            };
            input.push_str(input_closure.as_str());

            fields_output.push(input);
        }

        html! {
            <form id={self.id.clone()} onsubmit={&self.on_submit} onchange={&self.on_change}>
                {
                    fields_output.iter().map(|input|{

                        let ele = document.create_element("div").unwrap();
                        ele.set_class_name("field-wrapper");
                        ele.set_inner_html(input.as_str());

                        let node: Node = ele.into();

                        Html::VRef(node)

                    }).collect::<Html>()
                }
                <input type="submit" class="btn-primary mb-0"/>
            </form>
        }
    }

    pub fn new(id: String, context: T, submit_label: String) -> Self {
        let mut form_builder = Self {
            id,
            context,
            on_submit: Callback::<FocusEvent>::default(),
            on_change: Callback::<Event>::default(),
            submit_label,
            field_keys: vec![],
            keys: HashMap::new(),
        };

        form_builder.build_form();
        form_builder
    }

    pub fn convert_event_and_set(event: Event, cb: Callback<FieldValue>) {
        let target: Option<EventTarget> = event.target();

        let input = event.target().unwrap().value_of();

        let onbject_lookup = String::from(input.to_locale_string());

        if onbject_lookup.contains("HTMLSelect") {
            let input = &target
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                .unwrap();

            let id = input.get_attribute("name").unwrap();

            cb.emit(FieldValue {
                id,
                value: input.value(),
            });
        } else if onbject_lookup.contains("HTMLTextArea") {
            let input = &target
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
                .unwrap();

            let id = input.get_attribute("name").unwrap();

            cb.emit(FieldValue {
                id,
                value: input.value(),
            });
        } else {
            let input = &target
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();

            let id = input.get_attribute("name").unwrap();

            cb.emit(FieldValue {
                id,
                value: input.value(),
            });
        };
    }
}
