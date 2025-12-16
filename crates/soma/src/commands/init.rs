use std::fs;
use std::io::{self, Cursor};
use std::path::PathBuf;

use clap::Args;
use reqwest;
use tracing::debug;
use zip::ZipArchive;

use shared::error::CommonError;

#[derive(Args, Debug, Clone)]
pub struct InitParams {
    /// The template name to use (e.g., 'js-agent' for https://github.com/trysoma/js-agent-template)
    #[arg(short = 't', long = "template", required = true)]
    pub template: String,

    /// The directory to create and download the template into (relative to current directory)
    #[arg(required = true)]
    pub directory: PathBuf,
}

pub async fn cmd_init(params: InitParams) -> Result<(), CommonError> {
    debug!(
        "Initializing new project from template '{}' into '{}'",
        params.template,
        params.directory.display()
    );

    // Get the current directory and resolve the target path
    let current_dir = std::env::current_dir()?;
    let target_dir = current_dir.join(&params.directory);

    // Create the directory recursively if it doesn't exist
    if !target_dir.exists() {
        debug!("Creating directory: {}", target_dir.display());
        fs::create_dir_all(&target_dir).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to create directory {}: {}",
                target_dir.display(),
                e
            ))
        })?;
    } else {
        debug!("Directory already exists: {}", target_dir.display());
    }

    // Construct the GitHub repo URL and zip download URL
    let repo_name = format!("{}-template", params.template);
    let zip_url = format!("https://github.com/trysoma/{repo_name}/archive/refs/heads/main.zip");

    debug!("Downloading template from {}", zip_url);

    // Download the zip file
    let response = reqwest::get(&zip_url).await.map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!(
            "Failed to download template from {zip_url}: {e}"
        ))
    })?;

    if !response.status().is_success() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to download template: HTTP {} - {}. Please verify the template '{}' exists at https://github.com/trysoma/{}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown"),
            params.template,
            repo_name
        )));
    }

    let zip_bytes = response
        .bytes()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read response bytes: {e}")))?;

    debug!("Extracting template...");

    // Extract the zip file
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to open zip archive: {e}")))?;

    // GitHub zip archives have a root directory with format: {repo-name}-{branch}
    // We'll strip this root directory when extracting
    let root_prefix = format!("{repo_name}-main/");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to read file from archive: {e}"))
        })?;

        let file_path = file.name();

        // Skip files that don't start with the root prefix
        if !file_path.starts_with(&root_prefix) {
            continue;
        }

        // Remove the root prefix to get the relative path
        let relative_path = file_path.strip_prefix(&root_prefix).unwrap();

        // Skip empty paths (the root directory itself)
        if relative_path.is_empty() {
            continue;
        }

        let output_path = target_dir.join(relative_path);

        if file.is_dir() {
            // Create directories
            fs::create_dir_all(&output_path).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to create directory {}: {}",
                    output_path.display(),
                    e
                ))
            })?;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to create parent directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            // Extract file
            let mut output_file = fs::File::create(&output_path).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to create file {}: {}",
                    output_path.display(),
                    e
                ))
            })?;

            io::copy(&mut file, &mut output_file).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to write file {}: {}",
                    output_path.display(),
                    e
                ))
            })?;

            #[cfg(unix)]
            {
                // Preserve file permissions on Unix systems
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    let permissions = fs::Permissions::from_mode(mode);
                    fs::set_permissions(&output_path, permissions).map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to set permissions for {}: {}",
                            output_path.display(),
                            e
                        ))
                    })?;
                }
            }
        }
    }

    println!(
        "✓ Template '{}' successfully initialized in '{}'",
        params.template,
        target_dir.display()
    );
    println!("To get started, run:");
    println!("  cd {}", params.directory.display());

    Ok(())
}
