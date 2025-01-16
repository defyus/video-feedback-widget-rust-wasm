use actix_web::cookie::Cookie;
use rand::{thread_rng, Rng};

pub struct Utilities {}

impl Utilities {
    pub fn rnd_id(prefix: &str) -> String {
        let mut _prefix = String::from(prefix);

        let mut rng = thread_rng();
        let id = rng.gen_range(0..999999).to_string();

        _prefix.push_str(id.as_str());
        _prefix
    }

    pub fn get_cookie_value(cookies: &Vec<Cookie>, key: &'static str) -> String {
        for cookie in cookies.iter() {
            if key == cookie.name() {
                return cookie.value().to_string();
            }
        }

        return String::from("");
    }
}
