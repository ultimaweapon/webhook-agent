use serde::Deserialize;

/// Configuration for program.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub github_username: String,
}
