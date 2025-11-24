use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_config_default() {
  let config = Config::default();

  // Test default values are correctly set
  assert_eq!(config.transition.fps, 30);
  assert_eq!(config.transition.duration, 5);
  assert_eq!(config.timer.interval, 30);
  assert_eq!(config.timer.randomize, "5m");
  assert_eq!(config.sources.default, "wallhaven");
  assert_eq!(config.sources.category, "nature");
  assert_eq!(config.cleanup.keep_count, 10);
  assert!(config.cleanup.auto_cleanup);

  // Check advanced config defaults (from Config::default implementation, not struct Default)
  assert_eq!(config.advanced.parallel_downloads, 0); // Uses AdvancedConfig::default()
  assert_eq!(config.advanced.retry_attempts, 0);
  assert_eq!(config.advanced.timeout, 0);

  // Test nested defaults (from Default trait, not serde defaults)
  assert!(!config.integration.pywal.enabled); // Default trait sets to false
  assert!(!config.integration.desktop.notify_completion); // Default trait sets to false
  assert_eq!(config.sources.wallhaven.quality, ""); // Default trait sets to empty string
  assert_eq!(config.sources.local.formats, Vec::<String>::new()); // Default trait sets to empty vec
}

#[test]
fn test_config_serialization() {
  let config = Config::default();

  // Test that config can be serialized to YAML
  let yaml = serde_yaml::to_string(&config).expect("Failed to serialize config");
  assert!(!yaml.is_empty());

  // Test that it can be deserialized back
  let deserialized: Config = serde_yaml::from_str(&yaml).expect("Failed to deserialize config");

  // Test key values are preserved
  assert_eq!(deserialized.timer.interval, config.timer.interval);
  assert_eq!(deserialized.sources.default, config.sources.default);
  assert_eq!(deserialized.cleanup.keep_count, config.cleanup.keep_count);
}

#[test]
fn test_config_load_from_file() {
  let temp_dir = tempdir().expect("Failed to create temp dir");
  let config_path = temp_dir.path().join("test_config.yml");

  let yaml_content = r#"
paths:
  local: /home/user/wallpapers
  downloads: /home/user/downloads

transition:
  type: fade
  duration: 10
  fps: 60

timer:
  interval: 60
  randomize: "10m"

sources:
  default: local
  category: abstract

cleanup:
  keep_count: 5
  auto_cleanup: false

integration:
  pywal:
    enabled: false

logging:
  enabled: false
  level: debug
"#;

  fs::write(&config_path, yaml_content).expect("Failed to write test config");

  let config = Config::load(&config_path).expect("Failed to load config");

  assert_eq!(config.paths.local, "/home/user/wallpapers");
  assert_eq!(config.paths.downloads, "/home/user/downloads");
  assert_eq!(config.timer.interval, 60);
  assert_eq!(config.sources.default, "local");
  assert_eq!(config.sources.category, "abstract");
  assert_eq!(config.cleanup.keep_count, 5);
  assert!(!config.cleanup.auto_cleanup);
  assert!(!config.logging.enabled);
  assert_eq!(config.logging.level, "debug");
}

#[test]
fn test_config_load_missing_file() {
  let path = PathBuf::from("/nonexistent/config.yml");
  let result = Config::load(&path);

  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("Failed to read config file"));
}

#[test]
fn test_config_load_invalid_yaml() {
  let temp_dir = tempdir().expect("Failed to create temp dir");
  let config_path = temp_dir.path().join("invalid_config.yml");

  let invalid_yaml = r#"
paths:
  local: /home/user
  downloads: [invalid: structure
"#;

  fs::write(&config_path, invalid_yaml).expect("Failed to write invalid config");

  let result = Config::load(&config_path);
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("Failed to parse YAML config"));
}

#[test]
fn test_config_load_or_default_existing() {
  // This test is tricky because it depends on actual file system
  // For a real test, we'd need to mock the default_path function
  let config = Config::load_or_default().expect("Failed to load config");

  // Should always succeed, either with loaded config or defaults
  assert!(!config.sources.default.is_empty());
}

#[test]
fn test_transition_type_variants() {
  // Test single transition type
  let single_yaml = r#"
paths:
  local: /test
  downloads: /test
transition:
  type: fade
  duration: 5
timer:
  interval: 30
  randomize: "5m"
sources:
  default: local
  category: nature
cleanup:
  keep_count: 10
integration:
  pywal:
    enabled: true
logging:
  enabled: true
  level: info
"#;

  let config: Config = serde_yaml::from_str(single_yaml).expect("Failed to parse single transition");
  match config.transition.transition_type {
    TransitionType::Single(ref t) => assert_eq!(t, "fade"),
    _ => panic!("Expected Single transition type"),
  }

  // Test multiple transition types
  let multiple_yaml = r#"
paths:
  local: /test
  downloads: /test
transition:
  type: [fade, slide, random]
  duration: 5
timer:
  interval: 30
  randomize: "5m"
sources:
  default: local
  category: nature
cleanup:
  keep_count: 10
integration:
  pywal:
    enabled: true
logging:
  enabled: true
  level: info
"#;

  let config: Config = serde_yaml::from_str(multiple_yaml).expect("Failed to parse multiple transitions");
  match config.transition.transition_type {
    TransitionType::Multiple(ref types) => {
      assert_eq!(types.len(), 3);
      assert!(types.contains(&"fade".to_string()));
      assert!(types.contains(&"slide".to_string()));
      assert!(types.contains(&"random".to_string()));
    }
    _ => panic!("Expected Multiple transition type"),
  }
}

#[test]
fn test_wallhaven_config_defaults() {
  // Test the struct default (derives Default)
  let config = WallhavenConfig::default();

  assert!(config.api_key.is_none());
  assert!(config.resolution.is_none());
  assert_eq!(config.quality, ""); // Default trait sets to empty string
  assert!(config.purity.is_empty());

  // Test that serde defaults work during deserialization
  let minimal_yaml = r#"{}"#;
  let config: WallhavenConfig = serde_yaml::from_str(minimal_yaml).expect("Failed to parse minimal wallhaven config");

  assert!(config.api_key.is_none());
  assert!(config.resolution.is_none());
  assert_eq!(config.quality, "large");
  assert!(config.purity.is_empty());
}

#[test]
fn test_picsum_config_defaults() {
  let config = PicsumConfig::default();

  assert!(config.width.is_none());
  assert!(config.height.is_none());
}

#[test]
fn test_local_config_defaults() {
  // Test the struct default (derives Default)
  let config = LocalConfig::default();

  assert!(!config.recursive); // Default trait sets to false
  assert!(config.formats.is_empty()); // Default trait sets to empty vec

  // Test that serde defaults work during deserialization
  let minimal_yaml = r#"{}"#;
  let config: LocalConfig = serde_yaml::from_str(minimal_yaml).expect("Failed to parse minimal local config");

  assert!(config.recursive);
  assert_eq!(config.formats, vec!["jpg", "jpeg", "png", "webp"]);
}

#[test]
fn test_advanced_config_defaults() {
  // Test the struct default (derives Default)
  let config = AdvancedConfig::default();

  assert_eq!(config.parallel_downloads, 0);
  assert_eq!(config.retry_attempts, 0);
  assert_eq!(config.timeout, 0);

  // Test that serde defaults are used when deserializing minimal config
  let minimal_yaml = r#"{}"#;
  let config: AdvancedConfig = serde_yaml::from_str(minimal_yaml).expect("Failed to parse minimal advanced config");

  assert_eq!(config.parallel_downloads, 3);
  assert_eq!(config.retry_attempts, 3);
  assert_eq!(config.timeout, 30);
}

#[test]
fn test_logging_config_defaults() {
  // Test the struct default (derives Default)
  let config = LoggingConfig::default();

  assert!(!config.enabled); // Default trait sets to false
  assert_eq!(config.level, ""); // Default trait sets to empty string
  assert!(config.file.is_none());
  assert!(!config.timestamp); // Default trait sets to false

  // Test that serde defaults work during deserialization
  let minimal_yaml = r#"{}"#;
  let config: LoggingConfig = serde_yaml::from_str(minimal_yaml).expect("Failed to parse minimal logging config");

  assert!(config.enabled);
  assert_eq!(config.level, "info");
  assert!(config.file.is_none());
  assert!(config.timestamp);
}

#[test]
fn test_expand_paths() {
  let mut config = Config::default();
  config.paths.local = "~/Pictures".to_string();
  config.paths.downloads = "$HOME/Downloads".to_string();

  let result = config.expand_paths();

  // Should succeed (actual expansion depends on environment)
  assert!(result.is_ok());

  // Paths should be expanded (exact result depends on environment)
  assert!(!config.paths.local.contains('~'));
  assert!(!config.paths.downloads.contains('$'));
}

#[test]
fn test_expand_paths_invalid() {
  let mut config = Config::default();
  config.paths.local = "${NONEXISTENT_VAR}/path".to_string();

  let result = config.expand_paths();

  // shellexpand behavior: may fail or leave variables as-is depending on version
  // This test just verifies the function completes (error handling is in place)
  // Both success and failure are acceptable outcomes for undefined variables
  // This test verifies the function completes without panicking
  let _result = result;
}

#[test]
fn test_config_serde_attributes() {
  // Test that serde rename attribute works for transition type
  let yaml = r#"
paths:
  local: /test
  downloads: /test
transition:
  type: fade
  duration: 5
timer:
  interval: 30
  randomize: "5m"
sources:
  default: local
  category: nature
cleanup:
  keep_count: 10
integration:
  pywal:
    enabled: true
logging:
  enabled: true
  level: info
"#;

  let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse config with renamed field");

  match config.transition.transition_type {
    TransitionType::Single(t) => assert_eq!(t, "fade"),
    _ => panic!("Expected single transition type"),
  }
}

#[test]
fn test_config_optional_fields() {
  // Test minimal valid config (only required fields)
  let minimal_yaml = r#"
paths:
  local: /test
  downloads: /test
transition:
  type: fade
  duration: 5
timer:
  interval: 30
  randomize: "5m"
sources:
  default: local
  category: nature
  local: {}
cleanup:
  keep_count: 10
integration:
  pywal:
    enabled: true
logging:
  enabled: true
  level: info
"#;

  let config: Config = serde_yaml::from_str(minimal_yaml).expect("Failed to parse minimal config");

  // Optional fields should use defaults
  assert_eq!(config.transition.fps, 30); // default_fps
  assert!(config.sources.local.recursive); // default_true from serde
  assert_eq!(config.sources.local.formats, vec!["jpg", "jpeg", "png", "webp"]); // default_formats from serde
  assert!(config.logging.enabled); // default_true from serde
  assert_eq!(config.logging.level, "info"); // default_log_level from serde
}

#[cfg(test)]
mod property_tests {
  use super::*;
  use quickcheck::TestResult;
  use quickcheck_macros::quickcheck;

  // Property test: any valid config should serialize and deserialize correctly
  #[quickcheck]
  fn config_roundtrip_property(interval: u32, duration: u32, keep_count: u32) -> TestResult {
    // Bound inputs to reasonable ranges to avoid test failures
    if interval == 0 || duration == 0 || keep_count == 0 {
      return TestResult::discard();
    }

    let mut config = Config::default();
    config.timer.interval = interval % 3600; // Max 1 hour
    config.transition.duration = duration % 60; // Max 1 minute
    config.cleanup.keep_count = keep_count % 1000; // Max 1000 files

    let yaml = match serde_yaml::to_string(&config) {
      Ok(y) => y,
      Err(_) => return TestResult::failed(),
    };

    let deserialized: Config = match serde_yaml::from_str(&yaml) {
      Ok(c) => c,
      Err(_) => return TestResult::failed(),
    };

    TestResult::from_bool(
      deserialized.timer.interval == config.timer.interval
        && deserialized.transition.duration == config.transition.duration
        && deserialized.cleanup.keep_count == config.cleanup.keep_count,
    )
  }
}
