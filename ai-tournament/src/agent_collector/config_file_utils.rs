use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use tracing::{error, instrument};

pub fn get_all_configs(dir: &Path) -> anyhow::Result<HashMap<String, String>> {
    let config_file = collect_yaml(dir)?;
    let yaml = std::fs::read_to_string(config_file)?;
    let full_config = parse_yaml(&yaml)?;
    Ok(full_config.configs)
}

pub fn get_eval_config(dir: &Path) -> anyhow::Result<String> {
    let config_file = collect_yaml(dir)?;
    let yaml = std::fs::read_to_string(config_file)?;
    let full_config = parse_yaml(&yaml)?;
    let config_name = &full_config.eval;
    let config = full_config
        .configs
        .get(config_name)
        .context("unknown config '{config_name}'")?;
    Ok(config.clone())
}

pub fn get_args_from_config(config: &str) -> anyhow::Result<Vec<String>> {
    if config.contains("\"") || config.contains("'") || config.contains("`") {
        bail!("arguments should not contain any quote")
    }
    if config.is_empty() {
        // "".split(" ") == vec![""] : we should allow having an empty config
        return Ok(vec![]);
    }
    Ok(config.split(" ").map(String::from).collect())
}

struct ConfigFile {
    eval: String,
    configs: HashMap<String, String>,
}

fn parse_yaml(yaml: &str) -> anyhow::Result<ConfigFile> {
    let mut eval = None;
    let mut configs = HashMap::new();
    let mut in_configs = false;

    // .peekable() ? (to 'exit' in_config)
    for (i, line) in yaml.lines().enumerate() {
        let line = line.trim_end();

        // Skip empty lines and comments
        if line.trim().is_empty() || line.trim().starts_with("#") {
            continue;
        }

        if !in_configs {
            if let Some(value) = line.strip_prefix("eval:") {
                let value = value.trim();
                if value.is_empty() {
                    bail!("Line {}: 'eval' value missing", i + 1);
                }
                eval = Some(value.to_string());
            } else if line.starts_with("configs:") {
                in_configs = true;
            } else {
                bail!("Line {}: Expected 'eval:' or 'configs:' key", i + 1);
            }
        } else {
            // Inside configs list, expect lines like: '- key: "value"'
            let line = line.trim_start();
            if !line.starts_with('-') {
                bail!("Line {}: Expected list item starting with '-'", i + 1);
            }
            let rest = line[1..].trim_start();
            // Now expect key: "value"
            let colon_pos = rest
                .find(':')
                .context(format!("Line {}: Missing ':' in config item", i + 1))?;
            let key = rest[..colon_pos].trim();
            let value_part = rest[colon_pos + 1..].trim();

            // Value should start and end with double quotes
            if !value_part.starts_with('"') || !value_part[1..].find('"').is_some() {
                bail!("Line {}: Value must be quoted with double quotes", i + 1);
            }

            // Remove surrounding quotes (does not handle escaped quotes inside)
            let second_quote = 1 + value_part[1..].find('"').unwrap();
            let value = &value_part[1..second_quote];

            configs.insert(key.to_string(), value.to_string());
        }
    }

    let eval = eval.ok_or_else(|| anyhow::anyhow!("Missing 'eval' key"))?;

    Ok(ConfigFile { eval, configs })
}

#[instrument]
pub(super) fn check_dir_integrity(dir: &Path) -> anyhow::Result<()> {
    let metadata = match dir.metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Error reading directory: {}", e);
            bail!("error reading directory: {}", e);
        }
    };
    if !metadata.is_dir() {
        error!("Not a directory");
        bail!("not a directory");
    }
    let Ok(_) = std::fs::read_dir(dir) else {
        error!("error reading directory");
        bail!("error reading directory");
    };
    Ok(())
}

fn collect_yaml(dir: &Path) -> anyhow::Result<PathBuf> {
    check_dir_integrity(dir)?;

    let mut result: Option<PathBuf> = None;
    // Safety: `check_dir_integrity` tested that read_dir is ok
    for entry in std::fs::read_dir(dir).unwrap() {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        let Ok(name) = entry.file_name().into_string() else {
            bail!("name error: {:?}", entry.file_name());
        };
        if name.ends_with(".yml") || name.ends_with(".yaml") {
            if result.is_some() {
                bail!(
                    "two YAML files found: {} and {name}",
                    result.unwrap().to_str().unwrap()
                );
            }
            result = Some(entry.path());
        }
    }
    result.context("YAML not found")
}
