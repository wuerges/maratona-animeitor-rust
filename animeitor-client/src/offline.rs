//! Portable revelation snapshots. The executable is the matching client build;
//! the payload contains data and presentation preferences only.
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use base64::{engine::general_purpose::STANDARD, Engine};
use data::{configdata::SedeEntry, ContestFile, RunsFile};
use leptos::{mount::mount_to_body, prelude::*};
use serde::{Deserialize, Serialize};
mod browser;
mod css;
mod export;

use crate::views::{
    global_settings::{
        provide_offline_settings, use_global_settings, GlobalSettings, TeamSettings,
    },
    reveleitor::Revelation,
};

const PAYLOAD_ID: &str = "reveleitor-offline-data";

#[derive(Clone)]
pub(crate) struct OfflineExportInputs {
    pub contest: ContestFile,
    pub runs: RunsFile,
    pub sede: SedeEntry,
}

#[derive(Default)]
struct ExportState {
    generation: u64,
    inputs: Option<Arc<OfflineExportInputs>>,
}

/// Scoped to the contest screen. A superseded request cannot publish old runs
/// into the current screen's save controls.
#[derive(Clone, Copy)]
pub(crate) struct OfflineExportContext(RwSignal<ExportState>);

impl OfflineExportContext {
    pub fn new() -> Self {
        Self(RwSignal::new(ExportState::default()))
    }

    pub fn begin(&self) -> u64 {
        self.0.update(|state| {
            state.generation += 1;
            state.inputs = None;
        });
        self.0.with_untracked(|state| state.generation)
    }

    pub fn publish(&self, generation: u64, inputs: OfflineExportInputs) {
        self.0.try_update(|state| {
            if state.generation == generation {
                state.inputs = Some(Arc::new(inputs));
            }
        });
    }

    pub fn inputs(&self) -> Option<Arc<OfflineExportInputs>> {
        self.0.with(|state| state.inputs.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresentationSettings {
    mute: bool,
    autoplay: bool,
    show_audio_controls: bool,
    background_color: Option<String>,
    team_background_color: Option<String>,
    team_details: bool,
    team_settings: HashMap<String, TeamSettings>,
}

impl From<GlobalSettings> for PresentationSettings {
    fn from(s: GlobalSettings) -> Self {
        Self {
            mute: s.mute,
            autoplay: s.autoplay,
            show_audio_controls: s.show_audio_controls,
            background_color: s.background_color,
            team_background_color: s.team_background_color,
            team_details: s.team_details,
            team_settings: s.team_settings,
        }
    }
}

impl From<PresentationSettings> for GlobalSettings {
    fn from(s: PresentationSettings) -> Self {
        Self {
            mute: s.mute,
            autoplay: s.autoplay,
            show_audio_controls: s.show_audio_controls,
            background_color: s.background_color,
            team_background_color: s.team_background_color,
            team_details: s.team_details,
            team_settings: s.team_settings,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OfflineMedia {
    photos: HashMap<String, String>,
    sounds: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfflineSnapshot {
    version: u32,
    contest: ContestFile,
    runs: RunsFile,
    sede: SedeEntry,
    settings: PresentationSettings,
    #[serde(default)]
    media: OfflineMedia,
    #[serde(default)]
    omitted_assets: Vec<String>,
}

impl OfflineSnapshot {
    fn new(
        contest: ContestFile,
        mut runs: RunsFile,
        sede: SedeEntry,
        settings: GlobalSettings,
    ) -> Self {
        let contest = contest.filter_sede(&sede.into_sede());
        runs.filter_teams(&contest);
        let mut settings = PresentationSettings::from(settings);
        settings
            .team_settings
            .retain(|login, _| contest.teams.contains_key(login));
        Self {
            version: 1,
            contest,
            runs,
            sede,
            settings,
            media: OfflineMedia::default(),
            omitted_assets: vec![],
        }
    }

    fn decode(encoded: &str) -> Result<Self, String> {
        let bytes = STANDARD
            .decode(encoded.trim())
            .map_err(|_| "Invalid offline file encoding")?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "Invalid offline file data")?;
        if value.get("version").and_then(|v| v.as_u64()) != Some(1) {
            return Err("Unsupported offline file version".into());
        }
        serde_json::from_value(value).map_err(|e| format!("Invalid offline snapshot: {e}"))
    }
}

static MEDIA: OnceLock<OfflineMedia> = OnceLock::new();

pub fn media_location(login: &str, sound: bool) -> Option<String> {
    MEDIA.get().map(|media| {
        let (map, fallback) = if sound {
            (&media.sounds, "applause")
        } else {
            (&media.photos, "fake")
        };
        // Never allow a missing offline asset to fall through to the SDK.
        map.get(login)
            .or_else(|| map.get(fallback))
            .cloned()
            .unwrap_or_else(|| {
                if sound {
                    "data:audio/wav;base64,".into()
                } else {
                    "data:image/svg+xml,%3Csvg%20xmlns=%22http://www.w3.org/2000/svg%22/%3E".into()
                }
            })
    })
}

/// Returns true even for malformed payloads: an offline file must never fall
/// through to the online startup and attempt network requests.
pub fn mount_if_present() -> bool {
    let document = leptos::prelude::document();
    let Some(element) = document.get_element_by_id(PAYLOAD_ID) else {
        return false;
    };
    let decoded = OfflineSnapshot::decode(&element.text_content().unwrap_or_default());
    match decoded {
        Err(error) => mount_to_body(move || view! { <p role="alert">{error}</p> }),
        Ok(snapshot) => {
            let _ = MEDIA.set(snapshot.media);
            mount_to_body(move || {
                provide_offline_settings(snapshot.settings.into());
                let has_omissions = !snapshot.omitted_assets.is_empty();
                view! {
                    <crate::views::background_color::BackgroundColorValue override_color=None.into() />
                    <Revelation sede=Arc::new(snapshot.sede.into_sede()) runs_file=snapshot.runs contest=snapshot.contest />
                    <crate::views::settings_accordion::SettingsAccordion show_secret=false>
                        <Show when=move || has_omissions>
                        <div class="offline-report"><p>"Unavailable offline assets"</p>
                            <ul>{snapshot.omitted_assets.iter().map(|asset| view! { <li>{asset.clone()}</li> }).collect_view()}</ul>
                        </div>
                        </Show>
                    </crate::views::settings_accordion::SettingsAccordion>
                }
            });
        }
    }
    true
}

#[component]
pub fn SaveOffline(contest: ContestFile, runs: RunsFile, sede: SedeEntry) -> impl IntoView {
    let settings = use_global_settings();
    let (busy, set_busy) = signal(false);
    let (message, set_message) = signal(String::new());
    let save = move |_| {
        if busy.get_untracked() {
            return;
        }
        let snapshot = OfflineSnapshot::new(
            contest.clone(),
            runs.clone(),
            sede.clone(),
            settings.global.get_untracked(),
        );
        let mut sources = OfflineMedia::default();
        for login in snapshot
            .contest
            .teams
            .keys()
            .map(String::as_str)
            .chain(["fake"])
        {
            sources
                .photos
                .insert(login.into(), crate::api::team_photo_location(login));
        }
        for login in snapshot
            .contest
            .teams
            .keys()
            .map(String::as_str)
            .chain(["applause"])
        {
            sources
                .sounds
                .insert(login.into(), crate::api::team_sound_location(login));
        }
        set_busy.set(true);
        set_message.set("Preparing offline file…".into());
        leptos::task::spawn_local(async move {
            let result = browser::save(
                snapshot,
                sources,
                std::rc::Rc::new(move |message| {
                    let _ = set_message.try_set(message);
                }),
            )
            .await;
            let _ = set_message.try_set(match result {
                Ok(message) => message,
                Err(error) => format!("Could not save offline file: {error}. Please retry."),
            });
            let _ = set_busy.try_set(false);
        });
    };
    view! {
        <div class="offline-save">
            <button disabled=move || busy.get() on:click=save>"Save offline HTML"</button>
            <span id="offline-save-status" role="status">{move || message.get()}</span>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client_model::RevelationDriver;
    use serde_json::json;

    #[test]
    fn export_context_rejects_superseded_inputs() {
        Owner::new().with(|| {
            let context = OfflineExportContext::new();
            let snapshot = fixture();
            let inputs = OfflineExportInputs {
                contest: snapshot.contest,
                runs: snapshot.runs,
                sede: snapshot.sede,
            };
            let old = context.begin();
            context.publish(old, inputs.clone());
            assert!(context.0.with_untracked(|state| state.inputs.is_some()));
            let current = context.begin();
            assert!(context.0.with_untracked(|state| state.inputs.is_none()));
            context.publish(old, inputs.clone());
            assert!(context.0.with_untracked(|state| state.inputs.is_none()));
            context.publish(current, inputs);
            assert!(context.0.with_untracked(|state| state.inputs.is_some()));
        });
    }

    pub(super) fn fixture() -> OfflineSnapshot {
        let contest = serde_json::from_value(json!({
            "contest_name": "São Paulo </script>", "current_time": 300, "maximum_time": 300,
            "score_freeze_time": 240, "penalty_per_wrong_answer": 20, "number_problems": 1,
            "teams": {
                "a1": {"login":"a1", "name":"Team A", "escola":"School", "placement":1,"placement_global":1,"problems":{},"id":100},
                "a2": {"login":"a2", "name":"Team B", "escola":"School", "placement":2,"placement_global":2,"problems":{},"id":101},
                "b1": {"login":"b1", "name":"Outside", "escola":"School", "placement":3,"placement_global":3,"problems":{},"id":102}
            }
        })).unwrap();
        let runs = serde_json::from_value(json!({"runs": {
            "1": {"id":1,"order":1,"time":100,"team_login":"a1","prob":"A","answer":{"Yes":{"time":100,"is_first":false,"run_id":1}}},
            "2": {"id":2,"order":2,"time":250,"team_login":"a2","prob":"A","answer":{"Yes":{"time":250,"is_first":false,"run_id":2}}},
            "3": {"id":3,"order":3,"time":260,"team_login":"b1","prob":"A","answer":{"No":{"run_id":3}}}
        }})).unwrap();
        let sede =
            serde_json::from_value(json!({"name":"Site A", "codes":["^a"], "style":null})).unwrap();
        let settings = GlobalSettings {
            secret: Some("DO-NOT-EXPORT".into()),
            secret_enabled: true,
            ..Default::default()
        };
        OfflineSnapshot::new(contest, runs, sede, settings)
    }

    #[test]
    fn snapshot_filters_and_roundtrips_without_credentials() {
        let snapshot = fixture();
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(!json.contains("DO-NOT-EXPORT"));
        assert!(!json.contains("secret"));
        assert!(!json.contains("b1"));
        let decoded = OfflineSnapshot::decode(&STANDARD.encode(json)).unwrap();
        assert_eq!(decoded.contest.contest_name, snapshot.contest.contest_name);
        assert_eq!(decoded.runs.len(), 2);
        let restored: GlobalSettings = decoded.settings.into();
        assert!(!restored.secret_enabled);
        assert!(restored.secret.is_none());
    }

    #[test]
    fn snapshot_preserves_presentation_and_team_settings() {
        let fixture = fixture();
        let mut settings = GlobalSettings {
            mute: true,
            autoplay: true,
            show_audio_controls: false,
            background_color: Some("#123456".into()),
            team_background_color: Some("#abcdef".into()),
            team_details: true,
            secret_enabled: true,
            secret: Some("DO-NOT-EXPORT".into()),
            team_settings: [
                (
                    "a1".into(),
                    TeamSettings {
                        autoplay: Some(true),
                        volume: 37,
                    },
                ),
                (
                    "a2".into(),
                    TeamSettings {
                        autoplay: Some(false),
                        volume: 62,
                    },
                ),
                (
                    "b1".into(),
                    TeamSettings {
                        autoplay: None,
                        volume: 15,
                    },
                ),
            ]
            .into(),
        };
        let snapshot = OfflineSnapshot::new(
            fixture.contest,
            fixture.runs,
            fixture.sede,
            settings.clone(),
        );
        let encoded = STANDARD.encode(serde_json::to_vec(&snapshot).unwrap());
        let restored: GlobalSettings = OfflineSnapshot::decode(&encoded).unwrap().settings.into();
        settings.secret = None;
        settings.secret_enabled = false;
        settings.team_settings.remove("b1");
        assert_eq!(restored, settings);
    }

    #[test]
    fn malformed_and_unsupported_payloads_are_errors() {
        for payload in [
            "!".into(),
            STANDARD.encode("bad json"),
            STANDARD.encode(r#"{"version":2}"#),
            STANDARD.encode(r#"{"version":1}"#),
        ] {
            assert!(OfflineSnapshot::decode(&payload).is_err());
        }
    }

    fn standings(driver: &RevelationDriver) -> serde_json::Value {
        fn remove_ids(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Object(map) => {
                    map.remove("id");
                    for v in map.values_mut() {
                        remove_ids(v);
                    }
                }
                serde_json::Value::Array(values) => {
                    for v in values {
                        remove_ids(v);
                    }
                }
                _ => (),
            }
        }
        let mut value = serde_json::to_value(driver.contest()).unwrap();
        remove_ids(&mut value);
        value
    }

    #[test]
    fn restored_inputs_reveal_identically_and_always_restart_frozen() {
        let snapshot = fixture();
        let mut online = RevelationDriver::new(snapshot.contest.clone(), snapshot.runs.clone());
        let frozen = standings(&online);
        online.reveal_top_n(0).unwrap();
        let final_scores = standings(&online);
        assert_ne!(final_scores, frozen);
        let restored =
            OfflineSnapshot::decode(&STANDARD.encode(serde_json::to_vec(&snapshot).unwrap()))
                .unwrap();
        let mut offline = RevelationDriver::new(restored.contest, restored.runs);
        assert_eq!(standings(&offline), frozen);
        online.restart();
        for _ in 0..3 {
            online.reveal_step();
            offline.reveal_step();
            assert_eq!(standings(&offline), standings(&online));
        }
        online.back_one();
        offline.back_one();
        assert_eq!(standings(&offline), standings(&online));
        offline.restart();
        assert_eq!(standings(&offline), frozen);
        offline.reveal_top_n(0).unwrap();
        assert_eq!(standings(&offline), final_scores);
    }
}
