use data::configdata::Contest;
use serde::{Deserialize, Serialize};
use service::config_secret::Secret;

#[derive(Deserialize, Serialize, Debug)]
pub struct SedeSecret {
    pub name: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigSecret {
    pub secrets: Vec<SedeSecret>,
}

impl ConfigSecret {
    pub fn into_secret(&self, salt: Option<String>, sedes: &Contest) -> Secret {
        let salt = salt.unwrap_or_default();
        let sedes_by_secret = self
            .secrets
            .iter()
            .filter_map(|sede_secret| {
                let complete = format!("{}{}", salt, sede_secret.secret);
                sedes
                    .get_sede_nome_sede(&sede_secret.name)
                    .map(|sede| (complete, sede.clone()))
            })
            .collect();
        Secret { sedes_by_secret }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use data::configdata::{ConfigContest, SedeEntry};

    use super::*;

    #[test]
    fn test_config_patterns() {
        let sede = SedeEntry {
            name: "sede-name".into(),
            codes: serde_json::from_value(json!(["teambr", "teammx"])).unwrap(),
            ..SedeEntry::default()
        };

        let config_contest = ConfigContest {
            sedes: Some(vec![sede]),
            titulo: SedeEntry {
                name: "dummy".to_string(),
                ..SedeEntry::default()
            },
        };
        let contest = config_contest.into_contest();

        let config_secret = ConfigSecret {
            secrets: vec![SedeSecret {
                name: "sede-name".into(),
                secret: "key".into(),
            }],
        };
        let secret = config_secret.into_secret(None, &contest);

        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teambr$"),
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teammx$")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx$")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx$")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx")
        );

        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("tea#mbr$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("tea#mmx$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx")
        );

        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teamag")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teamag$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teamag$")
        );
    }
}
