use std::{path::PathBuf, process::Command};

use anyhow::Result;
use serde::Deserialize;
use url::Url;

fn main() {
    let raw = RawConfig::from_json(include_str!("pycharm.json")).unwrap();
    let config = Config::from_raw(raw).unwrap();
    let dockerfile = format!(
        r#"FROM fedora
RUN curl -L -o 1.tar.gz {} && \
echo "{} 1.tar.gz" | sha256sum -c --status && \
tar -xzf 1.tar.gz -C /opt
RUN dnf in -y {}
CMD {}"#,
        config.url,
        config.hash,
        {
            config
                .pkgs
                .iter()
                .map(|a| [a, " "].concat())
                .collect::<Vec<_>>()
                .concat()
        },
        PathBuf::from("/opt").join(config.entry).display()
    );
    std::fs::write("Dockerfile", dockerfile).unwrap();
    Command::new("podman")
        .args(["build", "-t", "pycharm", "."])
        .status()
        .unwrap();
    Command::new("podman")
        .args([
            "run",
            "--rm",
            "-e",
            "DISPLAY",
            "-v",
            "/tmp/.X11-unix:/tmp/.X11-unix",
            "--security-opt",
            "label=type:container_runtime_t",
            "pycharm",
        ])
        .spawn()
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    url: String,
    hash: String,
    entry: PathBuf,
    pkgs: Vec<String>,
}

impl RawConfig {
    fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[derive(Debug)]
struct Config {
    url: Url,
    hash: String,
    entry: PathBuf,
    pkgs: Vec<String>,
}

impl Config {
    fn from_raw(raw: RawConfig) -> Result<Self> {
        Ok(Self {
            url: Url::parse(&raw.url)?,
            hash: raw.hash,
            entry: {
                anyhow::ensure!(raw.entry.is_relative());
                raw.entry
            },
            pkgs: raw.pkgs,
        })
    }
}
