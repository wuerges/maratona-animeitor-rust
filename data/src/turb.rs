use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum Msg {
    Ready,
    Login,
    Logout,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clarification {
    pub question: String,
    pub answer: String,
    pub time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClarificationGroup {
    pub clarifications: Vec<Clarification>,
    pub name: String,
}

impl ClarificationGroup {
    pub fn len(&self) -> usize {
        self.clarifications.len()
    }

    pub fn new(key: String) -> Self {
        Self {
            name: key,
            clarifications: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClarificationSet {
    pub clars: BTreeMap<String, ClarificationGroup>,
}

impl ClarificationSet {
    pub fn new() -> Self {
        Self {
            clars: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &String) -> Option<&ClarificationGroup> {
        self.clars.get(key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunSet {
    pub runs: BTreeMap<i64, Run>,
}

impl RunSet {
    pub fn new() -> Self {
        Self {
            runs: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub id: i64,
    pub time: i64,
    pub problem: String,
    pub language: String,
    pub result: Ans,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Ans {
    Yes,
    No,
    Wait,
}
