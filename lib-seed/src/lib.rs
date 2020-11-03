
use seed::{prelude::*, *};

mod stepping;
mod requests;
mod automatic;
mod views;
mod runs;
mod timer;

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
                        "stepping" => stepping::start(root_element),
                        "automatic" => automatic::start(root_element),
                        "runspanel" => runs::start(root_element),
                        "timer" => timer::start(root_element),
                        s => log!("wrong app!:", s)
                    };
            }
        }

    }
}
