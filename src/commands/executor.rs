use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub fn start_command_stream(preview: &str, tx: mpsc::Sender<String>) {
    let parts = match shlex::split(preview) {
        Some(p) => p,
        None => {
            let _ = tx.send("Error: Unable to parse command preview".to_string());
            return;
        }
    };

    if parts.is_empty() {
        let _ = tx.send("No command to execute".to_string());
        return;
    }

    thread::spawn(move || {
        let mut child = match Command::new(&parts[0])
            .args(parts.iter().skip(1))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(format!("Failed to start command: {e}"));
                    return;
                }
            };

        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");
        
        let tx_out = tx.clone();
        let tx_err = tx.clone();

        // Stdout olvasása külön szálon
        let stdout_thread = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let _ = tx_out.send(l);
                }
            }
        });

        // Stderr olvasása külön szálon
        let stderr_thread = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let _ = tx_err.send(format!("ERR: {l}"));
                }
            }
        });

        // Megvárjuk, amíg a parancs végez
        let status = child.wait();
        let _ = stdout_thread.join();
        let _ = stderr_thread.join();

        // Itt már az eredeti 'tx'-et használjuk, ami megmaradt ebben a szálban
        match status {
            Ok(s) if s.success() => {
                let _ = tx.send("--- Command finished successfully ---".to_string());
            },
            Ok(s) => {
                let _ = tx.send(format!("--- Command failed with status: {} ---", s));
            },
            Err(e) => {
                let _ = tx.send(format!("--- Error waiting for command: {} ---", e));
            }
        }
    });
}

pub fn execute_preview(preview: &str) -> Result<String> {
    let parts = shlex::split(preview).context("Unable to parse command preview")?;
    let mut cmd = Command::new(&parts[0]);
    cmd.args(parts.iter().skip(1));
    let output = cmd.output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
