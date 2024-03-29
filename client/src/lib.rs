use seed::{prelude::*, *};

mod automatic;
mod countdown;
mod helpers;
mod navigation;
mod requests;
mod reveleitor;
mod runs;
mod sede;
mod teams;
mod timer;
mod views;

#[wasm_bindgen(start)]
pub fn start() {
    // let root_element = document()
    //     .get_element_by_id("app")
    //     .expect("`section` as a root element");

    let roots = document().get_elements_by_tag_name("maratona");

    for i in 0..roots.length() {
        match roots.get_with_index(i) {
            None => (),
            Some(root_element) => {
                match root_element.class_name().as_str() {
                    "navigation" => navigation::start(root_element),
                    "reveleitor" => reveleitor::start(root_element),
                    "automatic" => automatic::start(root_element),
                    "runspanel" => runs::start(root_element),
                    "timerpanel" => timer::start(root_element),
                    "sedepanel" => sede::start(root_element),
                    "teams" => teams::start(root_element),
                    "countdown" => countdown::start(root_element),
                    s => log!("wrong app!:", s),
                };
            }
        }
    }
}
