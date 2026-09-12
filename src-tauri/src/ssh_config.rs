use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::store::AppStore;

const MARK_START: &str = "# >>> ssh-bootstrap-manager managed block >>>";
const MARK_END: &str = "# <<< ssh-bootstrap-manager managed block <<<";

#[derive(Debug, Error)]
pub enum SSHConfigError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Validation Error: {0}")]
    Validation(String),
}

pub fn default_config_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".ssh");
    path.push("config");
    path
}

fn render_host_block(
    alias: &str,
    hostname: &str,
    username: &str,
    port: u16,
    identity_file: Option<&str>,
    proxy_jump: Option<&str>,
    extra_options: &str,
) -> String {
    let mut lines = vec![format!("Host {}", alias)];
    lines.push(format!("    HostName {}", hostname));
    if !username.is_empty() {
        lines.push(format!("    User {}", username));
    }
    lines.push(format!("    Port {}", port));
    if let Some(key) = identity_file {
        lines.push(format!("    IdentityFile {}", key));
    }
    if let Some(pj) = proxy_jump {
        lines.push(format!("    ProxyJump {}", pj));
    }
    for opt_line in extra_options.lines() {
        let opt_line = opt_line.trim();
        if !opt_line.is_empty() {
            lines.push(format!("    {}", opt_line));
        }
    }
    lines.join("\n")
}

pub fn generate_block(store: &AppStore) -> String {
    let mut blocks = Vec::new();

    let mut jump_hosts: Vec<_> = store.jump_hosts.values().collect();
    jump_hosts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    for jh in jump_hosts {
        let key_path = jh.key_name.as_ref().and_then(|k| store.keys.get(k).map(|key| key.private_key_path.as_str()));
        blocks.push(render_host_block(
            &jh.name,
            &jh.hostname,
            &jh.username,
            jh.port,
            key_path,
            None,
            &jh.extra_options,
        ));
    }

    let mut servers: Vec<_> = store.servers.values().collect();
    servers.sort_by(|a, b| a.alias.to_lowercase().cmp(&b.alias.to_lowercase()));

    for s in servers {
        let key_path = s.key_name.as_ref().and_then(|k| store.keys.get(k).map(|key| key.private_key_path.as_str()));
        blocks.push(render_host_block(
            &s.alias,
            &s.hostname,
            &s.username,
            s.port,
            key_path,
            s.jump_host.as_deref(),
            &s.extra_options,
        ));
    }

    blocks.join("\n\n")
}

fn check_duplicate_aliases(store: &AppStore) -> Result<(), SSHConfigError> {
    let mut seen = HashSet::new();
    
    for jh in store.jump_hosts.values() {
        if !seen.insert(&jh.name) {
            return Err(SSHConfigError::Validation(format!("Duplicate Host alias detected: '{}'", jh.name)));
        }
    }
    
    for s in store.servers.values() {
        if !seen.insert(&s.alias) {
            return Err(SSHConfigError::Validation(format!("Duplicate Host alias detected: '{}'", s.alias)));
        }
    }
    
    Ok(())
}

fn strip_managed_hosts(text: &str, store: &AppStore) -> String {
    let mut managed_aliases: HashSet<String> = HashSet::new();
    for jh in store.jump_hosts.values() {
        managed_aliases.insert(jh.name.clone());
    }
    for s in store.servers.values() {
        managed_aliases.insert(s.alias.clone());
    }

    let mut out_lines = Vec::new();
    let mut skip_current_block = false;

    for line in text.lines() {
        let stripped = line.trim();
        let lower = stripped.to_lowercase();
        
        if lower.starts_with("host ") || lower.starts_with("match ") {
            if lower.starts_with("match ") {
                skip_current_block = false;
            } else {
                let parts: Vec<&str> = stripped.split_whitespace().collect();
                let aliases: Vec<&str> = parts.iter().skip(1).cloned().collect();
                
                let unmanaged: Vec<&str> = aliases.into_iter().filter(|a| !managed_aliases.contains(*a)).collect();
                
                if unmanaged.is_empty() {
                    skip_current_block = true;
                    continue;
                } else {
                    skip_current_block = false;
                    let prefix_len = line.len() - line.trim_start().len() + parts[0].len();
                    let prefix = &line[..prefix_len];
                    out_lines.push(format!("{} {}", prefix, unmanaged.join(" ")));
                    continue;
                }
            }
        }
        
        if !skip_current_block {
            out_lines.push(line.to_string());
        }
    }

    let cleaned = out_lines.join("\n");
    let re = regex::Regex::new(r"\n{3,}").unwrap();
    re.replace_all(&cleaned, "\n\n").into_owned()
}

pub fn merge_into_full_config(existing_text: &str, store: &AppStore) -> Result<String, SSHConfigError> {
    check_duplicate_aliases(store)?;
    let block = generate_block(store);
    
    let header = "# This block is auto-generated by SSH Bootstrap Manager.\n\
                  # Do not edit by hand - manage servers and jump hosts from the app;\n\
                  # manual changes here will be overwritten on the next save.\n";
                  
    let body = if block.is_empty() {
        header.to_string()
    } else {
        format!("{}{}\n", header, block)
    };
    
    let managed = format!("{}\n{}{}", MARK_START, body, MARK_END);

    let re_managed = regex::RegexBuilder::new(&format!("(?s){}.*?{}", regex::escape(MARK_START), regex::escape(MARK_END)))
        .build()
        .unwrap();

    let manual_text = if re_managed.is_match(existing_text) {
        re_managed.replace(existing_text, "").into_owned()
    } else {
        existing_text.to_string()
    };
    
    let cleaned_manual = strip_managed_hosts(&manual_text, store);
    
    let sep = if cleaned_manual.trim().is_empty() { "" } else { "\n\n" };
    let mut new_text = format!("{}{}{}", cleaned_manual.trim_end(), sep, managed);
    
    if !new_text.ends_with('\n') {
        new_text.push('\n');
    }
    
    Ok(new_text)
}

fn validate(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();

    for line in text.lines() {
        let stripped = line.trim();
        if stripped.starts_with('#') || stripped.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = stripped.split_whitespace().collect();
        if !parts.is_empty() && parts[0].to_lowercase() == "host" {
            for alias in parts.iter().skip(1) {
                if !seen.insert(alias.to_string()) {
                    problems.push(format!("Duplicate Host entry: '{}'", alias));
                }
            }
        }
        
        if stripped.to_lowercase().starts_with("port ") {
            if let Some(value) = stripped.splitn(2, |c: char| c.is_whitespace()).nth(1) {
                let value = value.trim();
                if value.parse::<u16>().is_err() || value == "0" {
                    problems.push(format!("Invalid Port value: '{}'", value));
                }
            }
        }
    }

    problems
}

pub fn write_config(store: &AppStore, config_path: Option<&Path>) -> Result<String, SSHConfigError> {
    let path = config_path.map(PathBuf::from).unwrap_or_else(default_config_path);
    let existing = if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        "".to_string()
    };

    let new_text = merge_into_full_config(&existing, store)?;
    
    let problems = validate(&new_text);
    if !problems.is_empty() {
        return Err(SSHConfigError::Validation(problems.join("; ")));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        let mut backup_path = path.clone();
        backup_path.set_extension("bak");
        fs::write(&backup_path, &existing)?;
    }

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &new_text)?;
    fs::rename(&tmp_path, &path)?;
    
    // permissions on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = fs::metadata(&path).map(|m| m.permissions()) {
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
        }
    }

    Ok(new_text)
}
