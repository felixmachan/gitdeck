use anyhow::{Context, Result};
use std::process::Command;

pub fn execute_preview(preview: &str) -> Result<String> {
    let parts = shlex::split(preview).context("Unable to parse command preview")?;
    if parts.is_empty() {
        return Ok("No command to execute".to_string());
    }

    let mut cmd = Command::new(&parts[0]);
    cmd.args(parts.iter().skip(1));
    let output = cmd.output().context("Failed to execute git command")?;

    let mut text = String::new();
    if !output.stdout.is_empty() {
        text.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    if text.trim().is_empty() {
        text = if output.status.success() {
            "Command completed with no output".to_string()
        } else {
            format!("Command failed with status: {}", output.status)
        };
    }

    Ok(text)
}
