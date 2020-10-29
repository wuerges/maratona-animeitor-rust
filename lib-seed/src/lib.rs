
use seed::{prelude::*, *};

mod stepping;

#[wasm_bindgen(start)]
pub fn start() {

    let root_element = document()
        .get_element_by_id("app")
        .expect("`section` as a root element");

    log!("root element", root_element.class_name());


    match root_element.class_name().as_str() {
        "stepping" => stepping::stepping_start(),
        s => log!("wrong app!:", s)
    }

}
