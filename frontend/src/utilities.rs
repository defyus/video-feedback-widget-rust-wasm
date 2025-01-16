use js_sys::Math;
use std::collections::HashMap;
use web_sys::{FocusEvent, FormData, HtmlFormElement};
use yew::TargetCast;

pub struct Utilities;

impl Utilities {
    pub fn config(key: &str) -> String {
        let mut config: HashMap<String, String> = HashMap::new();

        config.insert(
            String::from("ws_url"),
            String::from("ws://127.0.0.1:9011/ws/"),
        );

        match config.get(key) {
            Some(val) => val.to_string(),
            None => String::from("InvalidConfigKey"),
        }
    }

    pub fn rnd_id_f64() -> f64 {
        Math::floor(Math::random() * (999999) as f64)
    }

    pub fn rnd_id(prefix: &str) -> String {
        let mut _prefix = String::from(prefix);
        let id = Math::floor(Math::random() * (9999999) as f64).to_string();
        _prefix.push_str(id.as_str());
        _prefix
    }

    pub fn form_data(event: FocusEvent, fields: Vec<String>) -> HashMap<String, String> {
        event.prevent_default();

        let mut map = HashMap::new();

        let form = event.target_dyn_into::<HtmlFormElement>().unwrap();

        let form_data = FormData::new_with_form(&form).unwrap();

        for field in fields {
            let value = form_data.get(field.as_str()).as_string();
            match value {
                Some(val) => {
                    map.insert(field, val);
                }
                None => {
                    log::error!("stuct property {:?} not found in FormData.", { field });
                }
            }
        }

        map
    }
    pub fn string_to_static_str(s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }
}
