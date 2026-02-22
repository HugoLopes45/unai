use std::io::Read as _;
use std::path::Path;

use serde::Deserialize;

use crate::error::{Result, UnaiError};

/// Maximum config file size. Configs larger than this are rejected before parsing.
const MAX_CONFIG_BYTES: u64 = 1024 * 1024; // 1 MiB

#[derive(Debug, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default)]
    pub rules: Vec<UserRule>,
    #[serde(default)]
    pub ignore: IgnoreConfig,
}

#[derive(Debug, Deserialize)]
pub struct UserRule {
    pub pattern: String,
    pub replacement: Option<String>,
    pub severity: Option<String>,
    pub message: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
pub struct IgnoreConfig {
    #[serde(default)]
    pub words: Vec<String>,
    // Glob patterns for files to skip entirely — wired in v0.4.0.
    #[serde(default)]
    #[allow(dead_code)]
    pub files: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        let mut file = std::fs::File::open(path).map_err(|source| UnaiError::FileRead {
            path: path.into(),
            source,
        })?;
        if file.metadata().map(|m| m.len()).unwrap_or(0) > MAX_CONFIG_BYTES {
            return Err(UnaiError::ConfigInvalid(
                "config file exceeds 1 MiB size limit".to_string(),
            ));
        }
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|source| UnaiError::FileRead {
                path: path.into(),
                source,
            })?;
        let config: Config = toml::from_str(&content).map_err(|source| UnaiError::ConfigParse {
            path: path.into(),
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_from_cwd() -> Result<Option<Config>> {
        let path = Path::new("unai.toml");
        match Config::load(path) {
            Ok(cfg) => Ok(Some(cfg)),
            Err(UnaiError::FileRead { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(UnaiError::ConfigInvalid(format!(
                "unsupported version {}",
                self.version
            )));
        }
        for rule in &self.rules {
            if rule.pattern.is_empty() || rule.pattern.trim().is_empty() {
                return Err(UnaiError::ConfigInvalid(
                    "rule pattern cannot be empty".to_string(),
                ));
            }
            if let Some(ref s) = rule.severity {
                match s.as_str() {
                    "critical" | "high" | "medium" | "low" => {}
                    _ => {
                        return Err(UnaiError::ConfigInvalid(format!(
                            "unknown severity '{}'; valid: critical, high, medium, low",
                            s
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // Shared lock for tests that mutate the process working directory.
    // Both `missing_file_returns_none` and `load_from_cwd_success` must hold
    // this lock to prevent races when Cargo runs tests in parallel.
    static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn write_temp_config(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn parse_minimal_config() {
        let f = write_temp_config("version = 1\n");
        let config = Config::load(f.path()).unwrap();
        assert_eq!(config.version, 1);
        assert!(config.rules.is_empty());
        assert!(config.ignore.words.is_empty());
        assert!(config.ignore.files.is_empty());
    }

    #[test]
    fn parse_rules_list() {
        let toml = r#"
version = 1

[[rules]]
pattern = "synergize"
replacement = "work together"
severity = "high"
message = "Corporate jargon"

[[rules]]
pattern = "robust"
enabled = false
"#;
        let f = write_temp_config(toml);
        let config = Config::load(f.path()).unwrap();
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].pattern, "synergize");
        assert_eq!(
            config.rules[0].replacement.as_deref(),
            Some("work together")
        );
        assert_eq!(config.rules[0].severity.as_deref(), Some("high"));
        assert_eq!(config.rules[1].pattern, "robust");
        assert!(!config.rules[1].enabled);
    }

    #[test]
    fn parse_ignore_section() {
        let toml = r#"
version = 1

[ignore]
words = ["robust", "comprehensive"]
files = ["docs/examples/**", "test/fixtures/**"]
"#;
        let f = write_temp_config(toml);
        let config = Config::load(f.path()).unwrap();
        assert_eq!(config.ignore.words, vec!["robust", "comprehensive"]);
        assert_eq!(
            config.ignore.files,
            vec!["docs/examples/**", "test/fixtures/**"]
        );
    }

    #[test]
    fn invalid_version_returns_error() {
        let f = write_temp_config("version = 99\n");
        let err = Config::load(f.path()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unsupported version 99"),
            "expected ConfigInvalid, got: {msg}"
        );
    }

    #[test]
    fn missing_file_returns_none() {
        // load_from_cwd looks for ./unai.toml; run from a temp dir where it won't exist.
        let _lock = CWD_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let result = Config::load_from_cwd();
        std::env::set_current_dir(original).unwrap();
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn empty_pattern_rejected() {
        let toml = "version = 1\n[[rules]]\npattern = \"\"\n";
        let f = write_temp_config(toml);
        let err = Config::load(f.path()).unwrap_err();
        assert!(err.to_string().contains("empty"), "got: {err}");
    }

    #[test]
    fn unknown_severity_rejected() {
        let toml = "version = 1\n[[rules]]\npattern = \"synergize\"\nseverity = \"ultra\"\n";
        let f = write_temp_config(toml);
        let err = Config::load(f.path()).unwrap_err();
        assert!(err.to_string().contains("unknown severity"), "got: {err}");
    }

    #[test]
    fn config_too_large_rejected() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        // Write 1 MiB + 1 byte
        let data = vec![b'#'; 1024 * 1024 + 1];
        f.write_all(&data).unwrap();
        let err = Config::load(f.path()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("1 MiB") || msg.contains("size limit") || msg.contains("too large"),
            "got: {msg}"
        );
    }

    // Whitespace-only patterns are also rejected — they would match on every line.
    #[test]
    fn whitespace_only_pattern_rejected() {
        let toml = "version = 1\n[[rules]]\npattern = \"   \"\n";
        let f = write_temp_config(toml);
        let err = Config::load(f.path()).unwrap_err();
        assert!(err.to_string().contains("empty"), "got: {err}");
    }

    // load_from_cwd success path — finds and loads a valid unai.toml from the working directory.
    #[test]
    fn load_from_cwd_success() {
        let _guard = CWD_LOCK.lock().unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        write_temp_config("version = 1\n")
            .persist(tmp.path().join("unai.toml"))
            .unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let result = Config::load_from_cwd();
        std::env::set_current_dir(original).unwrap();
        let cfg = result.unwrap().expect("should load the config");
        assert_eq!(cfg.version, 1);
    }
}
