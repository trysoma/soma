#[derive(Debug, Clone)]
pub struct BuildEvent {
    pub message: String,
}

pub fn print_command_output(status: &std::process::ExitStatus, output: &std::process::Output) {
    if status.success() {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            println!("cargo:warning=Build output: {line}");
        }
    } else {
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            println!("cargo:warning=Build error: {line}");
        }
    }
}
