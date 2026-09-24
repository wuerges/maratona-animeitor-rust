//! Incremental mutations validate candidates before publishing any changes.
use super::*;
use data::incremental::*;
use std::collections::HashSet;

fn invalid(message: impl Into<String>) -> StoreError {
    StoreError::InvalidValue(message.into())
}
fn missing() -> StoreError {
    StoreError::NotFound("event, contest, site or item".into())
}
fn nonempty_patch(empty: bool) -> Result<(), StoreError> {
    if empty {
        Err(invalid("patch must contain at least one field"))
    } else {
        Ok(())
    }
}
fn unique<'a>(mut values: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = HashSet::new();
    values.all(|value| seen.insert(value))
}
fn team_ids(event: &Event) -> Result<(), StoreError> {
    if !unique(event.teams.iter().map(|t| t.login.as_str())) {
        return Err(StoreError::Conflict(
            "duplicate team logins; repair the roster with a full replacement".into(),
        ));
    }
    Ok(())
}
fn problem_ids(event: &Event) -> Result<(), StoreError> {
    if !unique(event.problems.iter().map(String::as_str)) {
        return Err(StoreError::Conflict(
            "duplicate problem identifiers; repair with a full replacement".into(),
        ));
    }
    Ok(())
}
pub(super) fn state(event: &Event) -> EventState {
    EventState {
        name: event.name.clone(),
        problems: event.problems.clone(),
        teams: event.teams.clone(),
        score_freeze_time_seconds: event.score_freeze_time_seconds,
        penalty_seconds: event.penalty_seconds,
        time_seconds: event.time_seconds,
        salt: event.salt.clone(),
        photo_url_format: event.photo_url_format.clone(),
        sound_url_format: event.sound_url_format.clone(),
    }
}
fn same_name(a: &str, b: &str) -> Result<(), StoreError> {
    if a != b {
        Err(invalid("resource name cannot change"))
    } else {
        Ok(())
    }
}
fn change_codes(
    current: &[String],
    patch: CodesPatch,
) -> Result<(Vec<String>, RegexSet), StoreError> {
    nonempty_patch(patch.add.is_empty() && patch.remove.is_empty())?;
    if patch.add.iter().any(|p| patch.remove.contains(p)) {
        return Err(invalid("a pattern cannot be both added and removed"));
    }
    let mut result: Vec<_> = current
        .iter()
        .filter(|p| !patch.remove.contains(p))
        .cloned()
        .collect();
    for pattern in patch.add {
        if !result.contains(&pattern) {
            result.push(pattern);
        }
    }
    let compiled = compile_codes(&result)?;
    Ok((result, compiled))
}

impl Engine {
    pub async fn patch_event(
        &self,
        name: &str,
        patch: EventPatch,
        keep_runs: bool,
    ) -> Result<EventState, StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner.events.get_mut(name).ok_or_else(missing)?;
        nonempty_patch(patch.is_empty())?;
        let next = patch.apply(&state(event));
        same_name(&next.name, name)?;
        if !patch.teams.is_missing() {
            if !unique(next.teams.iter().map(|t| t.login.as_str())) {
                return Err(invalid("team logins must be unique"));
            }
            if !keep_runs
                && event.teams.iter().any(|team| {
                    !next.teams.iter().any(|t| t.login == team.login)
                        && event.runs.iter().any(|r| r.team_login == team.login)
                })
            {
                return Err(StoreError::Conflict(
                    "team has stored runs; use keep_runs=true to retain its runs while removing it"
                        .into(),
                ));
            }
        }
        if !patch.problems.is_missing() {
            if !unique(next.problems.iter().map(String::as_str)) {
                return Err(invalid("problem identifiers must be unique"));
            }
            if event
                .problems
                .iter()
                .any(|p| !next.problems.contains(p) && event.runs.iter().any(|r| &r.prob == p))
            {
                return Err(StoreError::Conflict(
                    "problem has stored runs; remove runs before removing the problem".into(),
                ));
            }
        }
        let timer_changed = next.time_seconds != event.time_seconds
            || next.score_freeze_time_seconds != event.score_freeze_time_seconds;
        event.problems = next.problems.clone();
        event.teams = next.teams.clone();
        event.salt = next.salt.clone();
        event.photo_url_format = next.photo_url_format.clone();
        event.sound_url_format = next.sound_url_format.clone();
        event.penalty_seconds = next.penalty_seconds;
        event.time_seconds = next.time_seconds;
        event.score_freeze_time_seconds = next.score_freeze_time_seconds;
        if timer_changed {
            event.publish_timer();
        }
        Ok(next)
    }
    pub async fn patch_contest(
        &self,
        event: &str,
        contest: &str,
        patch: ContestPatch,
    ) -> Result<ContestConfig, StoreError> {
        let mut inner = self.inner.write().await;
        let entry = inner
            .events
            .get_mut(event)
            .and_then(|e| e.contests.get_mut(contest))
            .ok_or_else(missing)?;
        nonempty_patch(patch.is_empty())?;
        let next = patch.apply(&entry.config);
        same_name(&next.name, contest)?;
        let codes = compile_codes(&next.codes)?;
        entry.config = next.clone();
        entry.codes = codes;
        Ok(next)
    }
    pub async fn patch_site(
        &self,
        event: &str,
        contest: &str,
        site: &str,
        patch: SitePatch,
    ) -> Result<SiteConfig, StoreError> {
        let mut inner = self.inner.write().await;
        let entry = inner
            .events
            .get_mut(event)
            .and_then(|e| e.contests.get_mut(contest))
            .and_then(|c| c.sites.get_mut(site))
            .ok_or_else(missing)?;
        nonempty_patch(patch.is_empty())?;
        let next = patch.apply(&entry.config);
        same_name(&next.name, site)?;
        let codes = compile_codes(&next.codes)?;
        entry.config = next.clone();
        entry.codes = codes;
        Ok(next)
    }
    pub async fn add_team(&self, event: &str, team: NewTeam) -> Result<TeamInfo, StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner.events.get_mut(event).ok_or_else(missing)?;
        team_ids(event)?;
        if team.login.is_empty() {
            return Err(invalid("login must not be empty"));
        }
        if event.teams.iter().any(|t| t.login == team.login) {
            return Err(StoreError::Conflict("team already exists".into()));
        }
        let team: TeamInfo = team.into();
        event.teams.push(team.clone());
        Ok(team)
    }
    pub async fn get_team(&self, event: &str, login: &str) -> Result<TeamInfo, StoreError> {
        let inner = self.inner.read().await;
        let event = inner.events.get(event).ok_or_else(missing)?;
        team_ids(event)?;
        event
            .teams
            .iter()
            .find(|t| t.login == login)
            .cloned()
            .ok_or_else(missing)
    }
    pub async fn patch_team(
        &self,
        event: &str,
        login: &str,
        patch: TeamPatch,
    ) -> Result<TeamInfo, StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner.events.get_mut(event).ok_or_else(missing)?;
        team_ids(event)?;
        let team = event
            .teams
            .iter_mut()
            .find(|t| t.login == login)
            .ok_or_else(missing)?;
        nonempty_patch(patch.is_empty())?;
        let next = patch.apply(team);
        *team = next.clone();
        Ok(next)
    }
    pub async fn remove_team(
        &self,
        event: &str,
        login: &str,
        keep_runs: bool,
    ) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner.events.get_mut(event).ok_or_else(missing)?;
        team_ids(event)?;
        let index = event
            .teams
            .iter()
            .position(|t| t.login == login)
            .ok_or_else(missing)?;
        if !keep_runs && event.runs.iter().any(|r| r.team_login == login) {
            return Err(StoreError::Conflict(
                "team has stored runs; use keep_runs=true to retain its runs while removing it"
                    .into(),
            ));
        }
        event.teams.remove(index);
        Ok(())
    }
    pub async fn add_problem(
        &self,
        event: &str,
        body: NewProblem,
    ) -> Result<Vec<String>, StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner.events.get_mut(event).ok_or_else(missing)?;
        problem_ids(event)?;
        if body.problem.is_empty() {
            return Err(invalid("problem must not be empty"));
        }
        if event.problems.contains(&body.problem) {
            return Err(StoreError::Conflict("problem already exists".into()));
        }
        event.problems.push(body.problem);
        Ok(event.problems.clone())
    }
    pub async fn remove_problem(&self, event: &str, problem: &str) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner.events.get_mut(event).ok_or_else(missing)?;
        problem_ids(event)?;
        let index = event
            .problems
            .iter()
            .position(|p| p == problem)
            .ok_or_else(missing)?;
        if event.runs.iter().any(|r| r.prob == problem) {
            return Err(StoreError::Conflict(
                "problem has stored runs; remove runs first".into(),
            ));
        }
        event.problems.remove(index);
        Ok(())
    }
    pub async fn patch_contest_codes(
        &self,
        event: &str,
        contest: &str,
        patch: CodesPatch,
    ) -> Result<ContestConfig, StoreError> {
        let mut inner = self.inner.write().await;
        let entry = inner
            .events
            .get_mut(event)
            .and_then(|e| e.contests.get_mut(contest))
            .ok_or_else(missing)?;
        let (codes, compiled) = change_codes(&entry.config.codes, patch)?;
        entry.config.codes = codes;
        entry.codes = compiled;
        Ok(entry.config.clone())
    }
    pub async fn patch_site_codes(
        &self,
        event: &str,
        contest: &str,
        site: &str,
        patch: CodesPatch,
    ) -> Result<SiteConfig, StoreError> {
        let mut inner = self.inner.write().await;
        let entry = inner
            .events
            .get_mut(event)
            .and_then(|e| e.contests.get_mut(contest))
            .and_then(|c| c.sites.get_mut(site))
            .ok_or_else(missing)?;
        let (codes, compiled) = change_codes(&entry.config.codes, patch)?;
        entry.config.codes = codes;
        entry.codes = compiled;
        Ok(entry.config.clone())
    }
}
