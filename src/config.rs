use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    pub thumbnail_size: (u32, u32),
    pub view_size: (u32, u32),
    pub output_dir: PathBuf,
}

impl Config {
    pub fn from_album(path: PathBuf) -> anyhow::Result<Config> {
        let config_path = path.join("photojawn.conf.yml");
        let content = fs::read(&config_path)
            .with_context(|| format!("Failed to read config from {}", config_path.display()))?;
        let cfg = serde_yml::from_slice(&content)
            .with_context(|| format!("Failed to parse config from {}", config_path.display()))?;
        Ok(cfg)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thumbnail_size: (256, 256),
            view_size: (1024, 768),
            output_dir: PathBuf::from("site"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::skel::make_skeleton;
    use mktemp::Temp;

    #[test]
    fn test_default() {
        let c = Config::default();
        assert_eq!(c.thumbnail_size, (256, 256));
        assert_eq!(c.output_dir, PathBuf::from("site"));
    }

    #[test]
    fn from_yaml() {
        // Empty YAML gives full default values
        let default_cfg = Config::default();
        let cfg: Config = serde_yml::from_str("").unwrap();
        assert_eq!(cfg, default_cfg);

        // Default values for any unspecified fields
        let cfg: Config = serde_yml::from_str("thumbnail_size: [1, 1]").unwrap();
        assert_ne!(cfg, default_cfg);
        assert_eq!(cfg.thumbnail_size, (1, 1));
        assert_eq!(cfg.view_size, default_cfg.view_size);
    }

    #[test]
    fn from_base_album() {
        let tmpdir = Temp::new_dir().unwrap();
        make_skeleton(&tmpdir).unwrap();

        let cfg = Config::from_album(tmpdir.to_path_buf()).unwrap();
        assert_eq!(cfg, Config::default());
    }
}
