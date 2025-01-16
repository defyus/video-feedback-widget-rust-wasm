use std::collections::HashSet;
use web_sys::ReadableStream;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

pub enum Msg {}

pub enum Request {
    OnMessageSubmission(String),
    OnMessageSubmissionComplete(Option<ReadableStream>),
}

pub struct FeedbackService {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
}
impl Agent for FeedbackService {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = Request;
    type Output = String;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        let mut skip_response: bool = false;
        match msg {
            Request::OnMessageSubmission(form_data) => {
                skip_response = true;
                self.on_msg_submission(form_data)
            }
            Request::OnMessageSubmissionComplete(_stream) => {}
        }

        if !skip_response {
            for sub in self.subscribers.iter().filter(|s| s.is_respondable()) {
                self.link.respond(*sub, String::from(""));
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
}

impl FeedbackService {
    pub fn on_msg_submission(&self, _form_data: String) {
        let _link = self.link.clone();
        //Feedback Submittion
    }
}
