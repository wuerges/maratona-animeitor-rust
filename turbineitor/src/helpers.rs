use maratona_animeitor_rust::{auth::UserKey, data};

use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::errors::Error;
use crate::models::*;
use crate::schema::*;
use crate::Params;

use std::collections::BTreeMap;
use std::time::SystemTime;

use sha2::{Digest, Sha256};

pub fn check_password(
    username_p: &str,
    password_p: &str,
    connection: &PgConnection,
    params: &Params,
) -> Option<UserKey> {
    use self::usertable::dsl::*;

    let digest = format!("{:x}", Sha256::digest(password_p.as_bytes()));

    usertable
        .filter(contestnumber.eq(params.contest_number))
        .filter(usersitenumber.eq(params.site_number))
        .filter(username.eq(username_p))
        .load::<Usertable>(connection)
        .expect("User not found")
        .first()
        .map(|u| {
            if u.userpassword == Some(digest) {
                Some(UserKey {
                    contest_number: u.contestnumber,
                    site_number: u.usersitenumber,
                    user_number: u.usernumber,
                })
            } else {
                None
            }
        })
        .flatten()
}

pub fn answer_from_code(c: i32, time: i64) -> data::Answer {
    match c {
        0 => data::Answer::Wait,
        1 => data::Answer::Yes(time),
        _ => data::Answer::No,
    }
}

pub fn to_run_tuple(
    r: &Runtable,
    letters: &BTreeMap<i32, String>,
    teams: &BTreeMap<i32, data::Team>,
) -> Option<data::RunTuple> {
    let time = r.rundatediff as i64 / 60;

    teams.get(&r.usernumber).map(|t| data::RunTuple {
        id: r.runnumber as i64,
        time,
        team_login: t.login.clone(),
        prob: letters.get(&r.runproblem).unwrap().clone(),
        answer: answer_from_code(r.runanswer, time),
    })
}

pub fn get_problem_letters(params: &Params, connection: &PgConnection) -> BTreeMap<i32, String> {
    use self::problemtable::dsl::*;
    let mut t = BTreeMap::new();
    for p in problemtable
        .filter(contestnumber.eq(params.contest_number))
        .load::<Problemtable>(connection)
        .expect("Error loading problem letters")
    {
        t.insert(p.problemnumber, p.problemname);
    }
    t
}

pub fn get_all_teams(params: &Params, connection: &PgConnection) -> Result<BTreeMap<i32, data::Team>, Error> {
    use self::usertable::dsl::*;

    let mut t = BTreeMap::new();

    for u in usertable
        .filter(contestnumber.eq(params.contest_number))
        .filter(usersitenumber.eq(params.site_number))
        .filter(usertype.eq("team"))
        .load::<Usertable>(connection)?
    {
        t.insert(
            u.usernumber,
            data::Team::new(&u.username, "", &u.userfullname),
        );
    }

    Ok(t)
}

pub fn get_all_runs(params: &Params, connection: &PgConnection) -> Result<data::RunsFile, Error> {
    use self::runtable::dsl::*;
    let letters = get_problem_letters(params, connection);
    let teams = get_all_teams(params, connection)?;

    let res: Vec<(i32, i32, i32, i32, i32)> = runtable
        .filter(contestnumber.eq(params.contest_number))
        .filter(runsitenumber.eq(params.site_number))
        .select((runnumber, rundatediff, usernumber, runproblem, runanswer))
        .load(connection)?;

    let runs = res
        .iter()
        .flat_map(|(id, time_large, team_id, prob_id, ans_id)| {
            teams.get(&team_id).map(|t| {
                let time = *time_large as i64 / 60;
                data::RunTuple {
                    id: *id as i64,
                    time,
                    team_login: t.login.clone(),
                    prob: letters.get(&prob_id).unwrap().clone(),
                    answer: answer_from_code(*ans_id, time),
                }
            })
        });

    Ok(data::RunsFile::new(runs.collect()))
}

pub fn get_contest_file(params: &Params, connection: &PgConnection) -> Result<data::ContestFile, Error> {
    use self::contesttable::dsl::*;

    let contest_opt = contesttable
        .find(params.contest_number)
        .load::<Contesttable>(connection)?;
    let contest = contest_opt.first().ok_or(Error::WrongContestNumber(params.contest_number, -1))?;

    let number_problems = get_problem_letters(params, connection).len();

    let teams = get_all_teams(params, connection)?;

    let current_time_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let contest_start_date = contest.conteststartdate as i64;
    let elapsed = current_time_now - contest_start_date;

    let contest_duration = contest.contestduration as i64;
    let current_time = contest_duration.min(elapsed);

    // println!("crazy times: {:?}", (current_time_now, contest_start_date, elapsed, contest_duration, current_time));

    let score_freeze_time = contest
        .contestlastmilescore
        .unwrap_or(contest.contestduration);
    // let score_freeze_time = contest.contestduration;
    Ok(data::ContestFile::new(
        contest.contestname.clone(),
        teams.values().cloned().collect(),
        current_time / 60,
        contest.contestduration as i64 / 60,
        score_freeze_time as i64 / 60,
        contest.contestpenalty as i64 / 60,
        number_problems,
    ))
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_login() {
        let params = Params {
            contest_number: 1,
            site_number: 1,
            secret: "".to_string(),
        };
        let c = establish_connection();
        println!(
            "testing: {:?}",
            super::check_password("admin", "boca", &c, &params)
        );
    }
}
