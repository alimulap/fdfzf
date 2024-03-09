use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{stdout, Write},
    path::{Path, PathBuf},
    process,
};

fn main() {
    let matches = clap::Command::new("fdfzf")
        .arg(
            clap::Arg::new("path")
                .help("The path to the directory")
                .index(1),
        )
        .arg(
            clap::Arg::new("type")
                .short('t')
                .long("type")
                .help("The type of the fd command")
                .value_parser(["d", "f"]),
        )
        .arg(
            clap::Arg::new("depth")
                .short('d')
                .long("depth")
                .help("The depth of the directory"),
        )
        .arg(
            clap::Arg::new("hidden")
                .short('H')
                .long("hidden")
                .action(clap::ArgAction::SetTrue)
                .help("Include hidden files and directories"),
        )
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("The path to the configuration file"),
        )
        .arg(
            clap::Arg::new("profile")
                .short('p')
                .long("profile")
                .help("Profile from config file"),
        )
        .get_matches();
    let config = fs::read_to_string(
        matches
            .get_one::<String>("config")
            .unwrap_or(&get_or_generate_default_config())
            .clone(),
    )
    .expect("Failed to read the configuration file");
    let config = toml::from_str::<Config>(&config).expect("Failed to parse the configuration file");
    let default_profile = config
        .profiles
        .as_ref()
        .and_then(|profiles| profiles.get("default").cloned());
    let custom_profile_name = matches.get_one::<String>("profile").cloned();
    let custom_profile = custom_profile_name.as_ref().and_then(|name| {
        config
            .profiles
            .and_then(|profiles| profiles.get(name).cloned())
    });
    let path_str = matches
        .get_one::<String>("path")
        .or(custom_profile
            .as_ref()
            .and_then(|profile| profile.path.clone())
            .as_ref())
        .or(default_profile
            .as_ref()
            .and_then(|profile| profile.path.clone())
            .as_ref())
        .unwrap_or(&"~".to_string())
        .clone();
    let path = expand_tilde(path_str.clone()).unwrap();
    if path.exists() {
        let depth = matches
            .get_one::<String>("depth")
            .or(custom_profile
                .as_ref()
                .and_then(|profile| profile.depth.clone())
                .as_ref())
            .or(default_profile
                .as_ref()
                .and_then(|profile| profile.depth.clone())
                .as_ref())
            .unwrap_or(&"4".to_string())
            .clone();
        let fd_type = matches
            .get_one::<String>("type")
            .or(custom_profile
                .as_ref()
                .and_then(|profile| profile.fd_type.clone())
                .as_ref())
            .or(default_profile
                .as_ref()
                .and_then(|profile| profile.fd_type.clone())
                .as_ref())
            .unwrap_or(&"d".to_string())
            .clone();
        let hidden = if matches
            .get_one("hidden")
            .or(custom_profile
                .as_ref()
                .and_then(|profile| profile.hidden)
                .as_ref())
            .or(default_profile
                .as_ref()
                .and_then(|profile| profile.hidden)
                .as_ref())
            .unwrap_or(&false)
            .clone()
        {
            "--hidden"
        } else {
            "--no-hidden"
        };
        let fd = process::Command::new("fd")
            .args(&[
                ".",
                "--type",
                fd_type.as_str(),
                "--max-depth",
                depth.as_str(),
                hidden,
                path.as_os_str().to_str().unwrap(),
            ])
            .stdout(process::Stdio::piped())
            .spawn()
            .expect("Failed to execute command");
        let fzf = process::Command::new("fzf")
            .stdin(fd.stdout.unwrap())
            .stdout(process::Stdio::piped())
            .spawn()
            .expect("Failed to execute command");
        let output = fzf.wait_with_output().expect("Failed to wait on child");
        if output.status.success() {
            let selected = String::from_utf8(output.stdout).expect("Invalid UTF-8");
            stdout()
                .write_all(selected.as_bytes())
                .expect("Failed to write to stdout");
        }
        //else {
        //    eprintln!("The command failed: {}", output.status);
        //}
    } else {
        eprintln!("The directory does not exist: {}", path.display());
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    profiles: Option<HashMap<String, Profile>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Profile {
    path: Option<String>,
    depth: Option<String>,
    fd_type: Option<String>,
    hidden: Option<bool>,
}

fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir();
    }
    dirs::home_dir().map(|mut h| {
        if h == Path::new("/") {
            // Corner case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}

fn get_or_generate_default_config() -> String {
    let path = expand_tilde("~/.config/fdfzf/config.toml").unwrap();
    if !Path::new(&path).exists() {
        let config = r#"[profiles.default]
path = "~"
depth = "4"
fd_type = "d"
hidden = false
"#;
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create directory");
        fs::write(&path, config).expect("Failed to write to file");
    }
    return path.to_string_lossy().to_string();
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::Config;

    #[test]
    fn test_config() {
        let config = r#"
            [profiles.default]
            path = "~"
            depth = "4"
            fd_type = "d"
            hidden = false
            "#;
        let config = toml::from_str::<Config>(config).expect("Failed to parse the configuration");
        assert_eq!(config.profiles.as_ref().unwrap().len(), 1);
        assert_eq!(config.profiles.unwrap().get("default").unwrap().path, Some("~".to_string()));
    }

    #[test]
    fn test_get_or_generate_default_config() {
        let path = super::get_or_generate_default_config();
        assert!(path.ends_with("config.toml"));
        assert!(path.starts_with("/"));
        fs::remove_file(path).expect("Failed to remove file");
    }
}
