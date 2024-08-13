use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct WindowScroll {
    y: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ControlMessage {
    WindowScroll(WindowScroll),
}
