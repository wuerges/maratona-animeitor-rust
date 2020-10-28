// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
//#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};
extern crate rand;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    // Model::default()

    vec![0, 1]
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
type Model = Vec<i64>;
// struct Model {
//     items : Vec<i64>
// }

// impl Model {
//     fn append(&mut self) {
//         self.items.push(self.items.len() as i64)
//     }
// }

// impl Default for Model {
//     fn default() -> Self {
//         Self { items : Vec::new() }
//     }

// }

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
#[derive(Copy, Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    Append,
    Shuffle,
    Sort,
    SortEnd
}

fn shuffle(v: &mut  Vec<i64> ) {
    use rand::thread_rng;
    use rand::seq::SliceRandom;

    let mut rng = thread_rng();
    v.shuffle(&mut rng);
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Append => model.push(model.len() as i64),
        Msg::Shuffle => {
            orders.perform_cmd(cmds::timeout(1000, || Msg::Sort));
            shuffle(model)
        },
        Msg::Sort => {
            orders.perform_cmd(cmds::timeout(1000, || Msg::SortEnd));
            model.sort();
        },
        Msg::SortEnd => {
            log!("sort ended!")
        }
    }
}

fn make_style(e : & i64, offset : i64) -> seed::Style {
    style!{
        St::Position => "absolute",
        St::Top => px(100 - offset*50 + e*50),
        St::Transition => "1s ease top",
        St::BorderStyle => "solid",
        St::BorderWidth => px(1),
        St::Padding => px(5),
        St::BorderColor => if *e!=0 { CSSValue::Ignored } else { "red".into() },
    }
}

// ------ ------
//     View
// ------ ------

// (Remove the line below once your `Model` become more complex.)
// #[allow(clippy::trivially_copy_pass_by_ref)]
// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    div![
        "This is a counter: ",
        C!["counter"],
        button!["+1", ev(Ev::Click, |_| Msg::Append),],
        button!["shuffle", ev(Ev::Click, |_| Msg::Shuffle),],
        button!["sort", ev(Ev::Click, |_| Msg::Sort),],
        model.iter().enumerate().map( |(i,e)| 
            div![
                id![i],
                make_style(e, 0),
                i,
                "->",
                e
            ]
        ),
        // div![
        //     id![1],
        //     "Up",
        //     make_style(model),
        // ],
        // div![
        //     id![2],
        //     "Down",
        //     make_style(&(model+1)),
        // ],
        // <div id=1 style=updown_style(self.value%2) >{ "Up" }</div>
        // <div id=2 style=updown_style((1+self.value)%2) >{ "Down" }</div>
    ]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);


}
