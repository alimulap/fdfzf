use std::{io::{stdout, Write}, path::{Path, PathBuf}, process};

fn main() {
    let matches = clap::Command::new("sdir")
        .arg(
            clap::Arg::new("path")
                .help("The path to the directory")
                .index(1)
        )
        .arg(
            clap::Arg::new("type")
                .short('t')
                .long("type")
                .help("The type of the fd command")
                .value_parser(["d", "f"])
        )
        .arg(
            clap::Arg::new("depth")
                .short('d')
                .long("depth")
                .help("The depth of the directory")
        ).get_matches();
    let path_str = matches.get_one::<String>("path").unwrap_or(&"~".to_string()).clone();
    let path = expand_tilde(path_str.clone()).unwrap();
    if path.exists() {
        let depth = matches.get_one::<&str>("depth").unwrap_or(&"4");
        let fd_type = matches.get_one::<String>("type").unwrap_or(&"d".to_string()).clone();
        let fd = process::Command::new("fd")
            .args(&[
                ".",
                "--type", fd_type.as_str(),
                "--max-depth", depth,
                path.as_os_str().to_str().unwrap()
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
            stdout().write_all(selected.as_bytes()).expect("Failed to write to stdout");
        } else {
            eprintln!("The command failed: {}", output.status);
        }
    } else {
        eprintln!("The directory does not exist: {}", path.display());
    }
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
