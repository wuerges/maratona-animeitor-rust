use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod data;

use data::*;

struct Model {
    link: ComponentLink<Self>,
    runsPanel: RunsPanel,
    contest: Contest
}

enum Msg {
    AddRun(RunTuple)
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            runsPanel : RunsPanel::empty(),
            contest : Contest::new(Vec::new(), 0, 100, 30, 20)
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddRun(t) => {
                self.runsPanel.add_run(&t);
                self.contest.add_run(t);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| 
                    Msg::AddRun(RunTuple::from_string("375971416299teambrbr3BN").unwrap()))>{ "+1" }
                </button>

                { 
                    for self.runsPanel.latest_n(10).into_iter().map( |t| html!{ <p>{t.time}</p> } ) 
                }
                // <p>{ self.value[0].name }</p>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
