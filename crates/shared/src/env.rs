use std::path::PathBuf;

use anyhow::Context;

fn travese_up_for_env_file(file_name: &str) -> Option<PathBuf> {
    let relative_workspace_root = PathBuf::from("./../../").join(file_name);

    if PathBuf::from(file_name).exists() {
        println!("Loading environment variables from: {file_name}");
        Some(PathBuf::from(file_name))
    } else if relative_workspace_root.exists() {
        println!(
            "Loading environment variables from: {}",
            relative_workspace_root.display()
        );
        Some(relative_workspace_root)
    } else {
        println!("No environment variables file found");
        None
    }
}

fn load_optional_env_file(file_name: Option<PathBuf>) {
    match file_name {
        Some(path) => {
            dotenv::from_filename(path)
                .ok()
                .context("Failed to load environment variables (.env)")
                .unwrap();
        }
        None => {
            println!("No environment variables file found");
        }
    };
}

pub fn load_optional_env_files() {
    let env_path = travese_up_for_env_file(".env");
    let env_secrets_path = travese_up_for_env_file(".env.secrets");

    load_optional_env_file(env_path);
    load_optional_env_file(env_secrets_path);
}

pub fn configure_env() -> Result<(), anyhow::Error> {
    load_optional_env_files();
    Ok(())
}
