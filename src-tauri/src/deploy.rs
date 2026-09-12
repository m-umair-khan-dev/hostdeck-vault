use ssh2::Session;
use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SSH Error: {0}")]
    Ssh(#[from] ssh2::Error),
    #[error("Authentication failed.")]
    AuthFailed,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
}

pub fn deploy_key(
    hostname: &str,
    username: &str,
    password: &str,
    public_key_path: &str,
    port: u16,
) -> Result<(), DeployError> {
    let tcp = TcpStream::connect(format!("{}:{}", hostname, port))
        .map_err(|e| DeployError::ConnectionFailed(e.to_string()))?;
        
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    sess.userauth_password(username, password)?;
    if !sess.authenticated() {
        return Err(DeployError::AuthFailed);
    }

    let public_key_text = fs::read_to_string(public_key_path)?;
    let public_key_text = public_key_text.trim();

    let commands = vec![
        "mkdir -p ~/.ssh && chmod 700 ~/.ssh".to_string(),
        format!("echo '{}' >> ~/.ssh/authorized_keys", public_key_text),
        "chmod 600 ~/.ssh/authorized_keys".to_string(),
    ];

    for cmd in commands {
        let mut channel = sess.channel_session()?;
        channel.exec(&cmd)?;
        
        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr)?;
        
        channel.wait_close()?;
        
        if !stderr.is_empty() {
            return Err(DeployError::CommandFailed(stderr));
        }
    }

    Ok(())
}

fn find_ssh_client() -> String {
    #[cfg(windows)]
    let candidates = vec!["C:\\Windows\\System32\\OpenSSH\\ssh.exe", "ssh"];
    #[cfg(not(windows))]
    let candidates = vec!["ssh"];

    for candidate in candidates {
        if let Ok(output) = Command::new(candidate).arg("-V").output() {
            if output.status.success() || String::from_utf8_lossy(&output.stderr).contains("OpenSSH") {
                return candidate.to_string();
            }
        }
    }
    "ssh".to_string()
}

pub fn test_passwordless(
    hostname: &str,
    username: &str,
    port: u16,
    identity_file: Option<&str>,
) -> Result<bool, DeployError> {
    let ssh_cmd = find_ssh_client();
    let mut cmd = Command::new(ssh_cmd);
    
    cmd.arg("-o").arg("StrictHostKeyChecking=no")
       .arg("-o").arg("BatchMode=yes")
       .arg("-o").arg("ConnectTimeout=5")
       .arg("-p").arg(port.to_string());
       
    if let Some(id_file) = identity_file {
        cmd.arg("-i").arg(id_file);
    }
    
    cmd.arg(format!("{}@{}", username, hostname))
       .arg("echo ok");
       
    let output = cmd.output()?;
    
    if output.status.success() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// ---------------- Tauri Commands ----------------

#[tauri::command]
pub fn tauri_deploy_key(
    hostname: String,
    username: String,
    password: String,
    public_key_path: String,
    port: u16,
) -> Result<(), String> {
    deploy_key(&hostname, &username, &password, &public_key_path, port)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tauri_test_passwordless(
    hostname: String,
    username: String,
    port: u16,
    identity_file: Option<String>,
) -> Result<bool, String> {
    test_passwordless(&hostname, &username, port, identity_file.as_deref())
        .map_err(|e| e.to_string())
}
