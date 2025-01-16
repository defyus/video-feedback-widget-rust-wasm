use yew::{html, Component, Context, Html, Properties};

#[derive(PartialEq, Properties)]
pub struct LoadingProp {
    pub load: bool,
}

pub enum Msg {}

pub struct Loading;

impl Component for Loading {
    type Message = Msg;
    type Properties = LoadingProp;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let _link = _ctx.link();

        match msg {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let _link = _ctx.link();

        html! {
        <div style={ if _ctx.props().load { "display:flex;"}else{"display:none;"}}
            class="w-full flex flex-col justify-center opacity-[.75] absolute top-0 bottom-[130px] items-center">
            <div class="loadingio-spinner-dual-ring-mj3m52aj1ah">
                <div class="ldio-w6ofkac7xen">
                    <div></div>
                    <div>
                        <div></div>
                    </div>
                </div>
            </div>
            <p class="text-white">{"Loading please wait."}</p>
        </div>
        }
    }
}
