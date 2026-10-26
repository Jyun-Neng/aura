use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "config.yaml";
const EXAMPLE_CONFIG_FILE: &str = "config.example.yaml";
// The committed sample database, and the gitignored working copy the server
// makes of it on first use, so running on sample data never modifies the
// committed file.
const SAMPLE_TEMPLATE: &str = "example/aura-example.db";
const SAMPLE_DATABASE: &str = "example/sample.db";

#[derive(serde::Deserialize)]
struct Config {
    database: String,
}

/// Reads `database` from config.yaml in the server crate's folder, wherever the
/// server is started from, falling back to config.example.yaml when there's no
/// config.yaml. A relative path is resolved against that folder.
pub fn database_path() -> Result<PathBuf, String> {
    let server_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut config_path = server_dir.join(CONFIG_FILE);
    if !config_path.is_file() {
        config_path = server_dir.join(EXAMPLE_CONFIG_FILE);
        println!(
            "No {} found; using {} (sample database). Copy it to {} to use your own.",
            CONFIG_FILE, EXAMPLE_CONFIG_FILE, CONFIG_FILE
        );
    }
    let text = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Can't read {}: {}", config_path.display(), e))?;
    let config: Config = serde_yaml_ng::from_str(&text)
        .map_err(|e| format!("Invalid {}: {}", config_path.display(), e))?;

    let mut path = expand_home(&config.database);
    if path.is_relative() {
        path = server_dir.join(path);
    }
    if !path.is_file() && path == server_dir.join(SAMPLE_DATABASE) {
        std::fs::copy(server_dir.join(SAMPLE_TEMPLATE), &path)
            .map_err(|e| format!("Can't create the sample database {}: {}", path.display(), e))?;
        println!("Created {} from {}.", SAMPLE_DATABASE, SAMPLE_TEMPLATE);
    }
    if !path.is_file() {
        return Err(format!(
            "Database not found at {}. Check that the folder is available, or fix `database` in {}.",
            path.display(),
            config_path.display()
        ));
    }
    Ok(path)
}

fn expand_home(path: &str) -> PathBuf {
    match (path.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => PathBuf::from(path),
    }
}
