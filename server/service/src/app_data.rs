use std::{collections::HashMap, sync::Arc};

use data::configdata::{ConfigContest, Contest, Sede};
use data::{ContestFile, RunTuple, RunsFile, TimerData};
use tokio::sync::{Mutex, broadcast};

use crate::config_secret::Secret;
use crate::contest_state::ContestState;
use crate::errors::ServiceResult;
use crate::remote_control::{ControlSender, create_remote_control};
use crate::{DB, RunsFileExt, dbupdate_v2, membroadcast};

/// Why an endpoint query was rejected.
#[derive(Debug)]
pub enum QueryError {
    /// The scoreboard is not live yet or the secret is invalid.
    Forbidden,
    /// The contest key does not exist.
    NotFound,
}

/// Shared state of the running server.
#[derive(Clone)]
pub struct AppData {
    shared_db: Arc<Mutex<DB>>,
    runs_tx: membroadcast::Sender<RunTuple>,
    time_tx: broadcast::Sender<TimerData>,
    config: Arc<HashMap<String, (ConfigContest, Contest, Secret)>>,
    remote_control: Arc<Mutex<HashMap<String, ControlSender>>>,
    server_api_key: Option<String>,
}

impl AppData {
    /// Builds the shared server state.
    ///
    /// Must be called from within a tokio runtime: when `boca_url` is given,
    /// the update loop is spawned as a detached task.
    pub fn new(
        config: HashMap<String, (ConfigContest, Contest, Secret)>,
        boca_url: Option<String>,
        server_api_key: Option<String>,
    ) -> Self {
        let config = Arc::new(config);

        let shared_db = Arc::new(Mutex::new(DB::empty()));
        let (runs_tx, _) = membroadcast::channel(1_000_000);
        let (time_tx, _) = broadcast::channel(1_000_000);

        let remote_control = Arc::new(Mutex::new(HashMap::new()));

        if let Some(url) = boca_url {
            let _update = tokio::task::spawn(dbupdate_v2::db_update_loop(
                url,
                shared_db.clone(),
                runs_tx.clone(),
                time_tx.clone(),
            ));
        }

        Self {
            shared_db,
            runs_tx,
            time_tx,
            config,
            remote_control,
            server_api_key,
        }
    }

    /// The contest filtered to the given site, if the scoreboard is live.
    pub async fn contest_file(&self, sede_config: &str) -> Result<ContestFile, QueryError> {
        let db = self.shared_db.lock().await;
        if db.time_file < 0 {
            return Err(QueryError::Forbidden);
        }

        match self.config.get(sede_config) {
            Some((_, contest, _)) => Ok(db.contest_file_begin.clone().filter_sede(&contest.titulo)),
            None => Err(QueryError::NotFound),
        }
    }

    /// The raw config for the given contest key, if the scoreboard is live.
    pub async fn config_contest(&self, sede_config: &str) -> Result<&ConfigContest, QueryError> {
        let db = self.shared_db.lock().await;
        if db.time_file < 0 {
            return Err(QueryError::Forbidden);
        }

        match self.config.get(sede_config) {
            Some((config, _, _)) => Ok(config),
            None => Err(QueryError::NotFound),
        }
    }

    /// The secret runs of the site unlocked by `secret`, if the scoreboard is live.
    pub async fn secret_runs(
        &self,
        sede_config: &str,
        secret: &str,
    ) -> Result<RunsFile, QueryError> {
        let sede = self
            .config
            .get(sede_config)
            .and_then(|(_, _, s)| s.get_sede_by_secret(secret));

        match sede {
            None => Err(QueryError::Forbidden),
            Some(sede) => {
                let db = self.shared_db.lock().await;
                if db.time_file < 0 {
                    Err(QueryError::Forbidden)
                } else {
                    Ok(db.run_file_secret.filter_sede(sede))
                }
            }
        }
    }

    /// The title site of the given contest key.
    pub fn sede_title(&self, sede_config: &str) -> Option<Sede> {
        self.config
            .get(sede_config)
            .map(|(_, contest, _)| contest.titulo.clone())
    }

    pub fn runs_subscribe(&self) -> membroadcast::Receiver<RunTuple> {
        self.runs_tx.subscribe()
    }

    pub fn time_subscribe(&self) -> broadcast::Receiver<TimerData> {
        self.time_tx.subscribe()
    }

    /// The broadcast channel of the remote-control key, creating it if needed.
    pub async fn remote_control_sender(&self, key: &str) -> ControlSender {
        let mut lock = self.remote_control.lock().await;

        lock.entry(key.to_string())
            .or_insert_with(create_remote_control)
            .clone()
    }

    pub async fn update_runs_from_data(&self, state: ContestState) -> ServiceResult<()> {
        dbupdate_v2::update_runs_from_data(state, &self.shared_db, &self.runs_tx, &self.time_tx)
            .await
    }

    pub fn server_api_key(&self) -> Option<&str> {
        self.server_api_key.as_deref()
    }
}
