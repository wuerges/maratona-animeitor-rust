use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;

extern crate rand;


fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    // orders.skip().perform_cmd( fetch_all() );
    orders.send_msg(Msg::Reset);
    Model { 
        url_filter : url.hash().map( |s| s.clone()),
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
        runs_queue : data::RunsQueue::empty(),
        // current_run: 0,
        center: None,
        // lock_frozen : true,
    }
}

struct Model {
    url_filter : Option<String>,
    contest : data::ContestFile,
    runs: data::RunsFile,
    runs_queue : data::RunsQueue,
    // current_run: usize,
    center : Option<String>,
    // lock_frozen : bool,
}

enum Msg {
    Prox(usize),
    Scroll(usize),
    Prox1,
    // Wait,
    // Recalculate,
    // ToggleFrozen,
    // FetchedRuns(fetch::Result<data::RunsFile>),
    // FetchedContest(fetch::Result<data::ContestFile>),
    Reset,
    Fetched(
        fetch::Result<data::RunsFile>,
        fetch::Result<data::ContestFile>),
}

async fn fetch_all() -> Msg {
    let r = fetch_allruns().await;
    let c = fetch_contest().await;
    Msg::Fetched(r, c)
}

// fn apply_run_model(model: &mut Model) {
//     if model.current_run < model.runs.runs.len() {
//         let mut run = model.runs.runs[model.current_run].clone();
//         run.answer = data::Answer::Wait;
//         model.contest.apply_run(&run).unwrap();
//     }
// }

fn apply_all_runs_before_frozen(model: &mut Model) {

    let mut mock_contest = model.contest.clone();

    for run in &model.runs.runs {
        if run.time < model.contest.score_freeze_time {
            model.contest.apply_run(run).unwrap();
            mock_contest.apply_run(run).unwrap();
        }
        else {
            let mut fake_run = run.clone();
            fake_run.answer = data::Answer::Wait;
            model.contest.apply_run(&fake_run).unwrap();

            let mut real_run = run.clone();
            // real_run.answer = data::Answer::Yes; // TODO fix this!
            real_run.answer = data::Answer::No; // TODO fix this!

            if mock_contest.useful_run(&real_run).unwrap() {
                mock_contest.apply_run(&real_run).unwrap();
                model.runs_queue.load_run(real_run);
            }

        }
    }
    model.runs_queue.setup_teams(&model.contest);
    // log!("after applying runs:", model.runs_queue.queue);
    model.contest.recalculate_placement().unwrap();
}

fn apply_one_run_from_queue(runs_queue: &mut data::RunsQueue, contest  : &mut data::ContestFile) {

    let r = runs_queue.pop_run(contest);
    log!("popped run:", r);

    // runs_queue.sort_by(|a, b| {
    //     let team_a = contest.teams.get(&a.team_login).unwrap();
    //     let team_b = contest.teams.get(&b.team_login).unwrap();

    //     team_a.placement.cmp(&team_b.placement)
    // });

    // match runs_queue.pop() {
    //     None => (),
    //     Some(current_run) => {
    //         contest.apply_run(&current_run).unwrap();
    //     }
    // }

    
}


fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        // Msg::ToggleFrozen => {
        //     model.lock_frozen = !model.lock_frozen;
        // },
        // Msg::Wait => {
        //     if model.current_run < model.runs.runs.len() {
        //         let mut run = model.runs.runs[model.current_run].clone();
        //         run.answer = data::Answer::Wait;
        //         model.contest.apply_run(&run).unwrap();
        //     }
        //     orders.perform_cmd(cmds::timeout(2000, move || Msg::Recalculate));
        // }
        // Msg::Recalculate => {
        //     if model.current_run < model.runs.runs.len() {
        //         let run = &model.runs.runs[model.current_run];
        //         model.contest.apply_run(run).unwrap();
        //         model.current_run += 1;
        //     }
        //     model.contest.recalculate_placement().unwrap();
        // },
        Msg::Prox1 => {
            apply_one_run_from_queue(&mut model.runs_queue, &mut model.contest);
            model.contest.recalculate_placement().unwrap();
        },
        Msg::Prox(n) => {
            model.center = model.runs_queue.queue.peek().map(|s| s.team_login.clone() );
            orders.perform_cmd(cmds::timeout(5000, move || Msg::Scroll(n)));
        },
        Msg::Scroll(n) => {
            model.center = None;

            while model.runs_queue.queue.len() > n {
                apply_one_run_from_queue(&mut model.runs_queue, &mut model.contest);
            }
            // for _ in 0..n {
            // }
            model.contest.recalculate_placement().unwrap();

        },
        Msg::Fetched(Ok(runs), Ok(contest)) => {
            // model.current_run = 0;
            model.center = None;
            model.runs = runs;
            model.runs.runs.reverse();
            model.contest = contest;
            apply_all_runs_before_frozen(model);
            model.contest.reload_score().unwrap();
            // log!("run queue: ", model.runs_queue);
        },
        Msg::Fetched(Err(e), _) => {
            log!("fetched runs error!", e)
        },
        Msg::Fetched(_, Err(e)) => {
            log!("fetched contest error!", e)
        },
        Msg::Reset => {
            orders.skip().perform_cmd( fetch_all() );    
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    // let frozen = if model.lock_frozen {"Frozen Locked"} else { "Frozen Unlocked"};
    div![
        div![
            C!["ccommandpanel"],
            button!["+1", ev(Ev::Click, |_| Msg::Prox1),],
            button!["Top 10", ev(Ev::Click, |_| Msg::Prox(10)),],
            button!["Top 30", ev(Ev::Click, |_| Msg::Prox(30)),],
            button!["Top 50", ev(Ev::Click, |_| Msg::Prox(50)),],
            button!["Top 100", ev(Ev::Click, |_| Msg::Prox(100)),],
            button!["Reset", ev(Ev::Click, |_| Msg::Reset),],
            // button![frozen, ev(Ev::Click, |_| Msg::ToggleFrozen),],
            div!["Missing teams: ", model.runs_queue.len()],
        ],
        views::view_scoreboard(&model.contest, &model.center, &model.url_filter),
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
