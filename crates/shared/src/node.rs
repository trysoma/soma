use std::{env, path::PathBuf};

/// Get the default fnm installation paths for different platforms
fn get_fnm_paths() -> Vec<PathBuf> {
    let home = env::var("HOME")
        .unwrap_or_else(|_| env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()));

    let mut paths = Vec::new();

    // Check if FNM_DIR is set (custom installation)
    if let Ok(fnm_dir) = env::var("FNM_DIR") {
        paths.push(PathBuf::from(fnm_dir));
    }

    // Default installation locations
    #[cfg(target_os = "macos")]
    {
        // macOS default locations
        paths.push(PathBuf::from(format!("{home}/.fnm")));
        paths.push(PathBuf::from(format!(
            "{home}/Library/Application Support/fnm"
        )));
    }

    #[cfg(target_os = "linux")]
    {
        // Linux default locations
        paths.push(PathBuf::from(format!("{home}/.fnm")));
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            paths.push(PathBuf::from(format!("{xdg_data_home}/fnm")));
        } else {
            paths.push(PathBuf::from(format!("{home}/.local/share/fnm")));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows default locations
        if let Ok(app_data) = env::var("APPDATA") {
            paths.push(PathBuf::from(format!("{app_data}\\fnm")));
        }
        paths.push(PathBuf::from(format!("{home}/.fnm")));
    }

    paths
}

/// Get the fnm binary paths that should be added to PATH
fn get_fnm_bin_paths() -> Vec<String> {
    let mut bin_paths = Vec::new();

    for base_path in get_fnm_paths() {
        // The default/current node version symlink
        let default_bin = base_path.join("aliases/default/bin");
        if default_bin.exists() {
            if let Some(path_str) = default_bin.to_str() {
                bin_paths.push(path_str.to_string());
            }
        }

        // The current-node symlink (if using fnm env)
        let current_bin = base_path.join("current/bin");
        if current_bin.exists() {
            if let Some(path_str) = current_bin.to_str() {
                bin_paths.push(path_str.to_string());
            }
        }
    }

    bin_paths
}

fn construct_path_env() -> String {
    let fnm_bin_paths = get_fnm_bin_paths();
    let mut path = String::new();
    for bin_path in fnm_bin_paths {
        path.push_str(&bin_path);
        path.push(':');
    }
    path
}

pub fn override_path_env() {
    let existing = std::env::var("PATH").unwrap_or_default();
    let new_path = construct_path_env();
    unsafe {
        std::env::set_var("PATH", format!("{new_path}:{existing}"));
    }
}
