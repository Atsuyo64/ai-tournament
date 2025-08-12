use std::{collections::HashMap, path::PathBuf};

use anyhow::{bail, Context};

pub fn get_all_configs(dir: &PathBuf) -> anyhow::Result<HashMap<String, String>> {
    let config_file = collect_pair(dir)?.1;
    let yaml = std::fs::read_to_string(config_file)?;
    let full_config = parse_yaml(&yaml)?;
    Ok(full_config.configs)
}

pub fn get_eval_config(dir: &PathBuf) -> anyhow::Result<String> {
    let config_file = collect_pair(dir)?.1;
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

        // Skip empty lines
        if line.trim().is_empty() {
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
            if !value_part.starts_with('"') || !value_part.ends_with('"') {
                bail!("Line {}: Value must be quoted with double quotes", i + 1);
            }

            // Remove surrounding quotes (does not handle escaped quotes inside)
            let value = &value_part[1..value_part.len() - 1];

            configs.insert(key.to_string(), value.to_string());
        }
    }

    let eval = eval.ok_or_else(|| anyhow::anyhow!("Missing 'eval' key"))?;

    Ok(ConfigFile { eval, configs })
}

pub(super) fn collect_pair(dir: &Path) -> anyhow::Result<(PathBuf, PathBuf)> {
    let mut result: (PathBuf, PathBuf) = Default::default();

    let Ok(metadata) = dir.metadata() else {
        bail!("error reading directory: {}", dir.metadata().unwrap_err());
    };
    if !metadata.is_dir() {
        bail!("not a directory");
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        bail!("error reading directory");
    };
    let cnt = entries.count();
    if cnt != 2 {
        bail!("directory contains {cnt} elements instead of 2");
    }

    let entries = std::fs::read_dir(dir).unwrap();
    let mut found = 0;
    for entry in entries {
        let Ok(entry) = entry else {
            bail!("one entry cannot be read in directory");
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            bail!("{:?} is not a file", entry.file_name());
        }
        let Ok(name) = entry.file_name().into_string() else {
            bail!("name error: {:?}", entry.file_name());
        };
        if name.ends_with(".yml") || name.ends_with(".yaml") {
            found |= 2;
            result.1 = entry.path();
        } else {
            found |= 1;
            result.0 = entry.path();
        }
    }
    if found == 0b11 {
        Ok(result)
    } else {
        bail!("missing directory directory content")
    }
}
