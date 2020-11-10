
use maratona_animeitor_rust::data;

use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::models::*;
use crate::schema::*;

use std::collections::BTreeMap;


pub fn answer_from_code(c:i32, time:usize) -> data::Answer {
    match c {
        0 => data::Answer::Wait,
        1 => data::Answer::Yes(time),
        _ => data::Answer::No,
    }
}

pub fn to_run_tuple(r : &Runtable, letters :&BTreeMap<i32, String>) -> data::RunTuple {
    
    
    let time = r.rundatediff as usize;
    data::RunTuple {
        id : r.runnumber as i64,
        time,
        team_login : "".to_string(),
        prob : letters.get(&r.runproblem).unwrap().clone(),
        answer : answer_from_code(r.runanswer, time),
    }
}

pub fn get_problem_letters(connection: &PgConnection) -> BTreeMap<i32, String> {
    use problemtable::dsl;
    
    let mut t = BTreeMap::new();
    for p in dsl::problemtable
    .load::<Problemtable>(connection)
    .expect("Error loading problem letters") {
        t.insert(p.problemnumber, p.problemname);
    }
    
    t
}

pub fn get_all_teams(connection: &PgConnection) -> BTreeMap<i32, data::Team> {
    use usertable::dsl;

    let mut t = BTreeMap::new();

    for u in dsl::usertable
    .load::<Usertable>(connection)
    .expect("Error loading users") {
        t.insert(u.usernumber, data::Team::new(
            &u.username,
            "",
            &u.userfullname,
        ));
    }

    t
}

pub fn get_all_runs(connection: &PgConnection) -> Vec<data::RunTuple> {
    use runtable::dsl;
    let letters = get_problem_letters(connection);

    dsl::runtable
        .load::<Runtable>(connection)
        .expect("Error loading runs")
        .iter()
        .map(|r| to_run_tuple(r, &letters))
        .collect()
}