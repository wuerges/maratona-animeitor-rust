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
    clarifications: Vec<Clarification>,
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

impl IntoIterator for ClarificationGroup {
    type Item = Clarification;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.clarifications.into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClarificationSet {
    clars: BTreeMap<String, ClarificationGroup>,
}

impl ClarificationSet {
    pub fn new() -> Self {
        Self {
            clars: BTreeMap::new(),
        }
    }

    pub fn get(&self, key :&String) -> &ClarificationGroup {
        self.clars.get(key).unwrap()
    }
}

impl IntoIterator for ClarificationSet {
    type Item = (String, ClarificationGroup);
    type IntoIter = std::collections::btree_map::IntoIter<String, ClarificationGroup>;

    fn into_iter(self) -> Self::IntoIter {
        self.clars.into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunSet {
    runs: BTreeMap<i64, Run>,
}

impl RunSet {
    pub fn new() -> Self {
        Self {
            runs: BTreeMap::new(),
        }
    }
}

impl IntoIterator for RunSet {
    type Item = (i64, Run);
    type IntoIter = std::collections::btree_map::IntoIter<i64, Run>;

    fn into_iter(self) -> Self::IntoIter {
        self.runs.into_iter()
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

// impl Iterator for ClarificationGroup {
//     // we will be counting with usize
//     type Item = usize;

//     // next() is the only required method
//     fn next(&mut self) -> Option<Self::Item> {
//         // Increment our count. This is why we started at zero.
//         self.count += 1;

//         // Check to see if we've finished counting or not.
//         if self.count < self.len() {
//             Some(self.)
//         } else {
//             None
//         }
//     }
// }
