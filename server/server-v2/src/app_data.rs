use std::{collections::HashMap, sync::Arc};

use data::{
    configdata::{ConfigContest, Contest, Secret},
    RunTuple, TimerData,
};
use service::{membroadcast, DB};
use tokio::sync::{broadcast, Mutex};

pub struct AppData {
    pub shared_db: Arc<Mutex<DB>>,
    pub runs_tx: membroadcast::Sender<RunTuple>,
    pub time_tx: broadcast::Sender<TimerData>,
    pub config: Arc<HashMap<String, (ConfigContest, Contest, Secret)>>,
}
