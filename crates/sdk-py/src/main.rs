use std::path::PathBuf;
use std::env;

fn main() {
    // Find the cdylib in common locations
    let lib_path = find_cdylib().expect("Could not find trysoma_sdk_core library. Build it first with `maturin build` or `cargo build`");

    println!("Introspecting library at: {}", lib_path.display());

    let module = pyo3_introspection::introspect_cdylib(&lib_path, "trysoma_sdk_core")
        .expect("Failed to introspect library");

    let result = pyo3_introspection::module_stub_files(&module);

    let stub_content = result
        .get(&PathBuf::from("__init__.pyi"))
        .expect("No __init__.pyi generated");

    // Add typing import if Callable is used in the stub
    let stub_content = if stub_content.contains("Callable") {
        format!("from typing import Callable\n\n{}", stub_content)
    } else {
        stub_content.clone()
    };

    if stub_content.is_empty() {
        eprintln!("Warning: pyo3-introspection generated empty stub file.");
        eprintln!("This may be because the module uses re-exports (pub use) which aren't introspectable.");
        eprintln!("Consider maintaining the stub file manually or adding #[pyo3(module = \"trysoma_sdk_core\")] to items.");
        std::process::exit(1);
    }

    // Determine output path - write directly to the installed package location
    // Since we need the .so to introspect, maturin must have already built and installed it
    let output_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Write to the same directory as the .so file we introspected
            lib_path.parent()
                .map(|p| p.join("__init__.pyi"))
                .unwrap_or_else(|| PathBuf::from("trysoma_sdk_core.pyi"))
        });

    std::fs::write(&output_path, &stub_content)
        .unwrap_or_else(|e| panic!("Failed to write stub file to {}: {}", output_path.display(), e));

    println!("Generated stub file at: {}", output_path.display());

    // Also write py.typed marker for PEP 561 compliance
    if let Some(parent) = output_path.parent() {
        let py_typed_path = parent.join("py.typed");
        if !py_typed_path.exists() {
            std::fs::write(&py_typed_path, "")
                .unwrap_or_else(|e| eprintln!("Warning: Failed to write py.typed marker: {}", e));
            println!("Created py.typed marker at: {}", py_typed_path.display());
        }
    }
}

fn find_cdylib() -> Option<PathBuf> {
    // Check for explicit path via environment variable
    if let Ok(path) = env::var("TRYSOMA_SDK_CORE_LIB") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    // Library name patterns to search for
    let lib_patterns = get_lib_patterns();

    // Common search paths
    let search_paths = get_search_paths();

    for base_path in search_paths {
        for pattern in &lib_patterns {
            let lib_path = base_path.join(pattern);
            if lib_path.exists() {
                return Some(lib_path);
            }
        }
    }

    None
}

fn get_lib_patterns() -> Vec<String> {
    let mut patterns = Vec::new();

    // ABI3 stable Python extension (maturin builds these)
    patterns.push("trysoma_sdk_core.abi3.so".to_string());

    #[cfg(target_os = "macos")]
    {
        // macOS uses .dylib for cdylib and .so for Python extensions
        patterns.push("trysoma_sdk_core.so".to_string());
        patterns.push("libtrysoma_sdk_core.dylib".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        patterns.push("trysoma_sdk_core.so".to_string());
        patterns.push("libtrysoma_sdk_core.so".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        patterns.push("trysoma_sdk_core.pyd".to_string());
        patterns.push("trysoma_sdk_core.dll".to_string());
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        patterns.push("trysoma_sdk_core.so".to_string());
        patterns.push("libtrysoma_sdk_core.so".to_string());
    }

    patterns
}

fn get_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Get the manifest directory (crates/sdk-py)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    // Project root (two levels up from crates/sdk-py)
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("../.."));

    // Python virtualenv site-packages (for maturin develop) - check these first as they have the correct .so
    if let Ok(virtual_env) = env::var("VIRTUAL_ENV") {
        let venv = PathBuf::from(virtual_env);
        // Try common Python versions
        for py_version in &["3.10", "3.11", "3.12", "3.13"] {
            paths.push(venv.join(format!("lib/python{}/site-packages/trysoma_sdk_core", py_version)));
        }
        // Windows style
        paths.push(venv.join("Lib/site-packages/trysoma_sdk_core"));
    }

    // py/.venv site-packages
    let py_venv = project_root.join("py/.venv");
    for py_version in &["3.10", "3.11", "3.12", "3.13"] {
        paths.push(py_venv.join(format!("lib/python{}/site-packages/trysoma_sdk_core", py_version)));
    }

    // Current working directory's py/.venv
    for py_version in &["3.10", "3.11", "3.12", "3.13"] {
        paths.push(PathBuf::from(format!("py/.venv/lib/python{}/site-packages/trysoma_sdk_core", py_version)));
    }

    // Cargo target directories (cargo build creates dylib here, but might not have PyInit)
    paths.push(project_root.join("target/debug"));
    paths.push(project_root.join("target/release"));
    paths.push(PathBuf::from("target/debug"));
    paths.push(PathBuf::from("target/release"));

    paths
}
