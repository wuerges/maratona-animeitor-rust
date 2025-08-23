use std::{collections::HashMap, sync::Arc};

use data::{
    RunTuple, TimerData,
    configdata::{ConfigContest, Contest, Secret},
};
use service::{DB, membroadcast};
use tokio::sync::{Mutex, broadcast};

use crate::remote_control;

pub struct AppData {
    pub shared_db: Arc<Mutex<DB>>,
    pub runs_tx: membroadcast::Sender<RunTuple>,
    pub time_tx: broadcast::Sender<TimerData>,
    pub config: Arc<HashMap<String, (ConfigContest, Contest, Secret)>>,
    pub remote_control: Arc<Mutex<HashMap<String, remote_control::ControlSender>>>,
    pub server_api_key: Option<String>,
}
