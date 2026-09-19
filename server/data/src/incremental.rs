//! Strict request types for atomic incremental configuration operations.
use crate::event::{ContestConfig, EventState, SiteConfig, TeamInfo};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

/// Distinguishes an omitted field from an explicitly supplied value (including null).
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Field<T> {
    #[default]
    Missing,
    Value(T),
}
impl<T> Field<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Field<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self::Value)
    }
}
impl<T: Serialize> Serialize for Field<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Missing => serializer.serialize_unit(),
            Self::Value(v) => v.serialize(serializer),
        }
    }
}

/// Selected EventState fields. Omitted fields are preserved; arrays are replaced. Empty objects are rejected.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EventPatch {
    /// Identifier must remain unchanged.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = String, required = false)]
    pub name: Field<String>,
    /// Replace the ordered problem list; removing referenced problems is a conflict.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Vec<String>, required = false)]
    pub problems: Field<Vec<String>>,
    /// Replace the roster; removing referenced teams requires keep_runs=true.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Vec<TeamInfo>, required = false)]
    pub teams: Field<Vec<TeamInfo>>,
    /// Inclusive freeze time in elapsed seconds.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = i64, required = false)]
    pub score_freeze_time_seconds: Field<i64>,
    /// Incorrect submission penalty in seconds.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = i64, required = false)]
    pub penalty_seconds: Field<i64>,
    /// Elapsed seconds, including negative countdown values.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = i64, required = false)]
    pub time_seconds: Field<i64>,
    /// Set salt; null clears it and changes affected revelation keys.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Option<String>, required = false)]
    pub salt: Field<Option<String>>,
}
impl EventPatch {
    pub fn is_empty(&self) -> bool {
        self.name.is_missing()
            && self.problems.is_missing()
            && self.teams.is_missing()
            && self.score_freeze_time_seconds.is_missing()
            && self.penalty_seconds.is_missing()
            && self.time_seconds.is_missing()
            && self.salt.is_missing()
    }
    pub fn apply(&self, current: &EventState) -> EventState {
        let mut next = current.clone();
        if let Field::Value(value) = &self.name {
            next.name = value.clone();
        }
        if let Field::Value(value) = &self.problems {
            next.problems = value.clone();
        }
        if let Field::Value(value) = &self.teams {
            next.teams = value.clone();
        }
        if let Field::Value(value) = &self.score_freeze_time_seconds {
            next.score_freeze_time_seconds = value.clone();
        }
        if let Field::Value(value) = &self.penalty_seconds {
            next.penalty_seconds = value.clone();
        }
        if let Field::Value(value) = &self.time_seconds {
            next.time_seconds = value.clone();
        }
        if let Field::Value(value) = &self.salt {
            next.salt = value.clone();
        }
        next
    }
}

/// Selected ContestConfig fields. Omitted fields are preserved; arrays are replaced. Empty objects are rejected.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ContestPatch {
    /// Identifier must remain unchanged.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = String, required = false)]
    pub name: Field<String>,
    /// Replace the complete regex filter list.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Vec<String>, required = false)]
    pub codes: Field<Vec<String>>,
    /// Set salt; null clears it.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Option<String>, required = false)]
    pub salt: Field<Option<String>>,
    /// Set style; null clears it.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Option<String>, required = false)]
    pub style: Field<Option<String>>,
    /// Inclusive gold placement threshold.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = usize, required = false)]
    pub ouro: Field<usize>,
    /// Inclusive silver placement threshold.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = usize, required = false)]
    pub prata: Field<usize>,
    /// Inclusive bronze placement threshold.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = usize, required = false)]
    pub bronze: Field<usize>,
    /// Photo template; null restores frontend default.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Option<String>, required = false)]
    pub photo_url_format: Field<Option<String>>,
    /// Audio template; null restores frontend default.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Option<String>, required = false)]
    pub sound_url_format: Field<Option<String>>,
}
impl ContestPatch {
    pub fn is_empty(&self) -> bool {
        self.name.is_missing()
            && self.codes.is_missing()
            && self.salt.is_missing()
            && self.style.is_missing()
            && self.ouro.is_missing()
            && self.prata.is_missing()
            && self.bronze.is_missing()
            && self.photo_url_format.is_missing()
            && self.sound_url_format.is_missing()
    }
    pub fn apply(&self, current: &ContestConfig) -> ContestConfig {
        let mut next = current.clone();
        if let Field::Value(value) = &self.name {
            next.name = value.clone();
        }
        if let Field::Value(value) = &self.codes {
            next.codes = value.clone();
        }
        if let Field::Value(value) = &self.salt {
            next.salt = value.clone();
        }
        if let Field::Value(value) = &self.style {
            next.style = value.clone();
        }
        if let Field::Value(value) = &self.ouro {
            next.ouro = value.clone();
        }
        if let Field::Value(value) = &self.prata {
            next.prata = value.clone();
        }
        if let Field::Value(value) = &self.bronze {
            next.bronze = value.clone();
        }
        if let Field::Value(value) = &self.photo_url_format {
            next.photo_url_format = value.clone();
        }
        if let Field::Value(value) = &self.sound_url_format {
            next.sound_url_format = value.clone();
        }
        next
    }
}

/// Selected SiteConfig fields. Omitted fields are preserved; arrays are replaced. Empty objects are rejected.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SitePatch {
    /// Identifier must remain unchanged.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = String, required = false)]
    pub name: Field<String>,
    /// Replace the complete regex filter list.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Vec<String>, required = false)]
    pub codes: Field<Vec<String>>,
    /// Set salt; null clears it.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = Option<String>, required = false)]
    pub salt: Field<Option<String>>,
}
impl SitePatch {
    pub fn is_empty(&self) -> bool {
        self.name.is_missing() && self.codes.is_missing() && self.salt.is_missing()
    }
    pub fn apply(&self, current: &SiteConfig) -> SiteConfig {
        let mut next = current.clone();
        if let Field::Value(value) = &self.name {
            next.name = value.clone();
        }
        if let Field::Value(value) = &self.codes {
            next.codes = value.clone();
        }
        if let Field::Value(value) = &self.salt {
            next.salt = value.clone();
        }
        next
    }
}

/// Selected TeamInfo fields. Omitted fields are preserved; arrays are replaced. Empty objects are rejected.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TeamPatch {
    /// Replace the displayed institution.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = String, required = false)]
    pub escola: Field<String>,
    /// Replace the displayed team name. Login is immutable.
    #[serde(default, skip_serializing_if = "Field::is_missing")]
    #[schema(value_type = String, required = false)]
    pub nome: Field<String>,
}
impl TeamPatch {
    pub fn is_empty(&self) -> bool {
        self.escola.is_missing() && self.nome.is_missing()
    }
    pub fn apply(&self, current: &TeamInfo) -> TeamInfo {
        let mut next = current.clone();
        if let Field::Value(value) = &self.escola {
            next.escola = value.clone();
        }
        if let Field::Value(value) = &self.nome {
            next.nome = value.clone();
        }
        next
    }
}

/// Exact regex-string additions and removals, applied atomically in order.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CodesPatch {
    /// Append patterns not already present, in request order.
    #[serde(default)]
    pub add: Vec<String>,
    /// Remove all occurrences of these exact strings; absent patterns are a no-op.
    #[serde(default)]
    pub remove: Vec<String>,
}
/// Append one problem identifier to the event's ordered list.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewProblem {
    pub problem: String,
}
/// Create one team. Unknown fields are rejected.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewTeam {
    pub login: String,
    pub escola: String,
    pub nome: String,
}
impl From<NewTeam> for TeamInfo {
    fn from(team: NewTeam) -> Self {
        Self {
            login: team.login,
            escola: team.escola,
            nome: team.nome,
        }
    }
}
