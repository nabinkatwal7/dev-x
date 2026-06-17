use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;

use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};

fn run_cmd(program: &str, args: &[&str]) -> Result<String, AppError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| AppError::Internal(format!("failed to execute '{}': {}", program, e)))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !stderr.trim().is_empty() {
            return Ok(stderr);
        }
    }
    Ok(stdout)
}

fn run_cmd_checked(program: &str, args: &[&str]) -> Result<String, AppError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| AppError::Internal(format!("failed to execute '{}': {}", program, e)))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        let detail = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
        return Err(AppError::Internal(format!("'{}' failed: {}", program, detail)));
    }
    Ok(if stdout.trim().is_empty() { stderr.trim().to_string() } else { stdout })
}

fn pid_exists(pid: u32) -> bool {
    if let Ok(output) = Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains(&pid.to_string())
    } else {
        false
    }
}

fn run_powershell(script: &str) -> Result<String, AppError> {
    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| AppError::Internal(format!("failed to execute powershell: {}", e)))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout.trim().to_string())
}

fn get_process_name(pid: u32) -> String {
    if pid == 0 {
        return "System Idle Process".into();
    }
    if pid == 4 {
        return "System".into();
    }
    if let Ok(output) = Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(name) = stdout.split(',').next() {
            let name = name.trim_matches('"').trim();
            if !name.is_empty() && name != "INFO" {
                return name.to_string();
            }
        }
    }
    format!("PID:{}", pid)
}

fn hosts_file_path() -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    format!("{}\\System32\\drivers\\etc\\hosts", root)
}

// ============================================================
// 011 - Zero-Latency Process Port Mapper
// ============================================================
pub fn list_listeners(filter: &str) -> Result<CommandExecutionResult, AppError> {
    let raw = run_cmd("netstat", &["-ano"])?;

    struct Entry {
        proto: String,
        local: String,
        state: String,
        pid: u32,
    }

    let mut entries: Vec<Entry> = Vec::new();
    let mut pids: Vec<u32> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("Active")
            || trimmed.starts_with("Proto")
        {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let proto = parts[0].to_string();
        let local = parts[1].to_string();
        let pid_str = parts.last().unwrap_or(&"");
        let pid: u32 = pid_str.parse().unwrap_or(0);
        let state = if proto == "TCP" && parts.len() >= 5 {
            parts[3].to_string()
        } else {
            "-".to_string()
        };

        if !filter.is_empty() {
            let foreign = parts[2].to_string();
            let display = format!("{} {} {} {}", proto, local, foreign, state);
            if !display.to_lowercase().contains(&filter.to_lowercase()) {
                continue;
            }
        }

        if pid > 0 && !pids.contains(&pid) {
            pids.push(pid);
        }
        entries.push(Entry { proto, local, state, pid });
    }

    let pid_names: Vec<(u32, String)> = pids.iter().map(|&pid| (pid, get_process_name(pid))).collect();

    let mut output = String::new();
    output.push_str(&format!(
        "{:<6} {:<42} {:<18} {:<8}  {}\n",
        "Proto", "Local Address", "State", "PID", "Process"
    ));
    output.push_str(&"-".repeat(120));
    output.push('\n');

    for e in &entries {
        let name = pid_names
            .iter()
            .find(|(p, _)| *p == e.pid)
            .map(|(_, n)| n.as_str())
            .unwrap_or("");
        output.push_str(&format!(
            "{:<6} {:<42} {:<18} {:<8}  {}\n",
            e.proto, e.local, e.state, e.pid, name
        ));
    }

    let summary = if filter.is_empty() {
        format!("Found {} active listener(s)", entries.len())
    } else {
        format!("Found {} listener(s) matching '{}'", entries.len(), filter)
    };

    Ok(CommandExecutionResult {
        command_id: "net.port-mapper".into(),
        title: "Port Mapper".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary,
    })
}

// ============================================================
// 012 - Atomic Process Termination
// ============================================================
fn list_processes(filter: &str) -> Result<String, AppError> {
    let output = run_cmd("tasklist", &["/FO", "CSV", "/NH", "/V"])?;
    let mut lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();

    let filter_lower = filter.to_lowercase();
    if !filter_lower.is_empty() && filter_lower != "list" {
        lines.retain(|l| l.to_lowercase().contains(&filter_lower));
    }

    if lines.is_empty() {
        return Ok("No matching processes found.".into());
    }

    let mut table = String::from(" PID       Name                          Session\n");
    table.push_str(" -------  -----------------------------  ------------------------------\n");

    for line in lines {
        let csv = line.trim().trim_matches('\"');
        let cols: Vec<&str> = csv.split("\",\"").collect();
        if cols.len() >= 8 {
            let name = cols[0].trim_matches('"');
            let pid = cols[1].trim_matches('"');
            let session = cols[7].trim_matches('"');
            table.push_str(&format!(" {:<8}  {:<29}  {}\n", pid, name, session));
        }
    }

    table.push_str("\nType a PID to kill it, or `list <filter>` to narrow results.");
    Ok(table)
}

pub fn kill_process(input: &str) -> Result<CommandExecutionResult, AppError> {
    let input = input.trim();
    if input.is_empty() || input.eq_ignore_ascii_case("list") {
        let output = list_processes("")?;
        return Ok(CommandExecutionResult {
            command_id: "net.kill-process".into(),
            title: "Running Processes".into(),
            output,
            status: CommandExecutionStatus::Info,
            summary: "Enter a PID to terminate, or `list <name>` to filter.".into(),
        });
    }

    if input.to_lowercase().starts_with("list ") {
        let filter = input[5..].trim();
        let output = list_processes(filter)?;
        return Ok(CommandExecutionResult {
            command_id: "net.kill-process".into(),
            title: if filter.is_empty() { "Running Processes".into() } else { format!("Processes: {}", filter) },
            output,
            status: CommandExecutionStatus::Info,
            summary: format!("Filtered by '{}'", if filter.is_empty() { "(none)" } else { filter }),
        });
    }

    let parts: Vec<&str> = input.split_whitespace().collect();
    let pid: u32 = parts[0]
        .parse()
        .map_err(|_| AppError::Internal(format!("invalid PID: '{}'. Enter a numeric PID or `list` to see processes.", parts[0])))?;
    let graceful = parts.len() > 1 && (parts[1] == "graceful" || parts[1] == "g" || parts[1] == "warn");

    let name = get_process_name(pid);

    if !pid_exists(pid) {
        return Ok(CommandExecutionResult {
            command_id: "net.kill-process".into(),
            title: "Process Not Found".into(),
            output: format!("PID {} ({}) does not exist.", pid, name),
            status: CommandExecutionStatus::Info,
            summary: "No such process".into(),
        });
    }

    if graceful {
        // Try graceful first, then fall back to force
        let _ = run_cmd_checked("taskkill", &["/PID", &pid.to_string()]);
        if !pid_exists(pid) {
            return Ok(CommandExecutionResult {
                command_id: "net.kill-process".into(),
                title: "Process Terminated (Graceful)".into(),
                output: format!("Process '{}' (PID: {}) terminated gracefully.", name, pid),
                status: CommandExecutionStatus::Success,
                summary: format!("PID {} ({}) terminated gracefully", pid, name),
            });
        }
        // Graceful failed — auto-escalate
        let msg = run_cmd_checked("taskkill", &["/F", "/T", "/PID", &pid.to_string()])?;
        let still_alive = pid_exists(pid);
        let output = if still_alive {
            format!(
                "Process '{}' (PID: {}) refused to die even with /F.\n{}",
                name, pid, msg
            )
        } else {
            format!(
                "Graceful kill failed, escalated to force kill.\nProcess '{}' (PID: {}) and its subprocesses terminated.\n{}",
                name, pid, msg
            )
        };
        let status = if still_alive {
            CommandExecutionStatus::Error
        } else {
            CommandExecutionStatus::Success
        };
        Ok(CommandExecutionResult {
            command_id: "net.kill-process".into(),
            title: if still_alive { "Process Could Not Be Terminated".into() } else { "Process Terminated (Force)".into() },
            output,
            status,
            summary: if still_alive { format!("PID {} ({}) still running — access denied", pid, name) } else { format!("PID {} ({}) terminated", pid, name) },
        })
    } else {
        // Force kill entire process tree
        let result = run_cmd_checked("taskkill", &["/F", "/T", "/PID", &pid.to_string()]);
        match result {
            Ok(msg) => {
                let still_alive = pid_exists(pid);
                let output = if still_alive {
                    format!(
                        "Force kill reported success but PID {} ({}) still exists.\n{}",
                        pid, name, msg
                    )
                } else {
                    format!(
                        "Process '{}' (PID: {}) and all subprocesses forcefully terminated.\n{}",
                        name, pid, msg
                    )
                };
                Ok(CommandExecutionResult {
                    command_id: "net.kill-process".into(),
                    title: if still_alive { "Process Still Alive".into() } else { "Process Terminated".into() },
                    output,
                    status: if still_alive { CommandExecutionStatus::Error } else { CommandExecutionStatus::Success },
                    summary: if still_alive {
                        format!("PID {} ({}) still running — try running as Administrator", pid, name)
                    } else {
                        format!("PID {} ({}) terminated (process tree)", pid, name)
                    },
                })
            }
            Err(e) => {
                // taskkill failed entirely (likely access denied)
                let still_alive = pid_exists(pid);
                if still_alive {
                    Ok(CommandExecutionResult {
                        command_id: "net.kill-process".into(),
                        title: "Termination Failed".into(),
                        output: format!(
                            "Could not terminate PID {} ({}).\nError: {}\n\nThis process may require Administrator privileges.",
                            pid, name, e
                        ),
                        status: CommandExecutionStatus::Error,
                        summary: format!("Access denied — PID {} ({}) still running", pid, name),
                    })
                } else {
                    // Process died between our check and the kill attempt
                    Ok(CommandExecutionResult {
                        command_id: "net.kill-process".into(),
                        title: "Process Already Terminated".into(),
                        output: format!("PID {} ({}) exited before the kill signal was sent.", pid, name),
                        status: CommandExecutionStatus::Success,
                        summary: format!("PID {} ({}) no longer exists", pid, name),
                    })
                }
            }
        }
    }
}

// ============================================================
// 013 - Multiplexed Port Monitor
// ============================================================
pub fn check_port(input: &str) -> Result<CommandExecutionResult, AppError> {
    let port_str = input.trim();
    if port_str.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "net.port-monitor".into(),
            title: "Port Monitor".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Enter a port number (e.g. 3000, 8080) to check if it is in use.".into(),
        });
    }

    let port: u16 = port_str
        .parse()
        .map_err(|_| AppError::Internal(format!("invalid port number: '{}'", port_str)))?;

    let raw = run_cmd("netstat", &["-ano"])?;
    let mut found = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Active") || trimmed.starts_with("Proto") {
            continue;
        }
        if trimmed.contains(&format!(":{}", port)) {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let pid_str = parts.last().unwrap_or(&"");
            let pid: u32 = pid_str.parse().unwrap_or(0);
            let name = get_process_name(pid);
            found.push(format!("{}  PID:{}  {}", trimmed, pid, name));
        }
    }

    if found.is_empty() {
        // Also try connecting to verify
        let reachable = TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", port).parse().unwrap(),
            Duration::from_secs(1),
        )
        .is_ok();

        if reachable {
            found.push(format!("Port {} is open but not showing in netstat output", port));
        }

        if found.is_empty() {
            return Ok(CommandExecutionResult {
                command_id: "net.port-monitor".into(),
                title: "Port Available".into(),
                output: format!("Port {} is not in use by any process.", port),
                status: CommandExecutionStatus::Success,
                summary: format!("Port {} is free", port),
            });
        }
    }

    let output = format!(
        "Port {} is IN USE by the following:\n\n{}",
        port,
        found.join("\n")
    );

    Ok(CommandExecutionResult {
        command_id: "net.port-monitor".into(),
        title: "Port In Use".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("Port {} is occupied by {} process(es)", port, found.len()),
    })
}

// ============================================================
// 014 - Local DNS Host File Editor
// ============================================================
pub fn read_hosts() -> Result<CommandExecutionResult, AppError> {
    let path = hosts_file_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Internal(format!("cannot read hosts file '{}': {}", path, e)))?;

    let mut parsed = String::new();
    let mut active = 0;
    let mut commented = 0;

    parsed.push_str(&format!("Hosts File: {}\n\n", path));
    parsed.push_str(&format!("{:<20} {:<40} {}\n", "IP Address", "Hostname", "Status"));
    parsed.push_str(&"-".repeat(80));
    parsed.push('\n');

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            let entry = trimmed.trim_start_matches('#').trim();
            if !entry.is_empty() && !entry.starts_with(' ') {
                let parts: Vec<&str> = entry.split_whitespace().collect();
                if parts.len() >= 2 && parts[0].contains('.') {
                    commented += 1;
                    parsed.push_str(&format!("{:<20} {:<40} [DISABLED]\n", parts[0], parts[1]));
                    continue;
                }
            }
            parsed.push_str(&format!("{:<62} [COMMENT]\n", trimmed));
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            active += 1;
            parsed.push_str(&format!("{:<20} {:<40} [ACTIVE]\n", parts[0], parts[1]));
        } else {
            parsed.push_str(&format!("{:<62} [RAW]\n", trimmed));
        }
    }

    let summary = format!("{} active entries, {} disabled, file: {}", active, commented, path);

    Ok(CommandExecutionResult {
        command_id: "net.hosts-edit".into(),
        title: "Hosts File Viewer".into(),
        output: parsed,
        status: CommandExecutionStatus::Success,
        summary,
    })
}

pub fn edit_hosts(input: &str) -> Result<CommandExecutionResult, AppError> {
    let path = hosts_file_path();
    let input = input.trim();

    if input.is_empty() || input == "list" || input == "view" {
        return read_hosts();
    }

    let parts: Vec<&str> = input.split_whitespace().collect();
    let action = parts[0].to_lowercase();

    match action.as_str() {
        "add" | "insert" => {
            if parts.len() < 3 {
                return Ok(CommandExecutionResult {
                    command_id: "net.hosts-edit".into(),
                    title: "Hosts Editor".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: "Usage: add <IP> <hostname> [hostname2...]".into(),
                });
            }
            let ip = parts[1];
            let hostnames = &parts[2..];
            let entry = format!("{} {}\n", ip, hostnames.join(" "));
            std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .map_err(|e| AppError::Internal(format!("cannot write hosts file: {}", e)))?
                .write_all(entry.as_bytes())
                .map_err(|e| AppError::Internal(format!("cannot write hosts file: {}", e)))?;
            Ok(CommandExecutionResult {
                command_id: "net.hosts-edit".into(),
                title: "Hosts Entry Added".into(),
                output: format!("Added to {}:\n{}", path, entry.trim()),
                status: CommandExecutionStatus::Success,
                summary: format!("Added {} -> {}", ip, hostnames.join(", ")),
            })
        }
        "remove" | "delete" | "rm" => {
            if parts.len() < 2 {
                return Ok(CommandExecutionResult {
                    command_id: "net.hosts-edit".into(),
                    title: "Hosts Editor".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: "Usage: remove <hostname>".into(),
                });
            }
            let target = parts[1];
            let content = std::fs::read_to_string(&path)
                .map_err(|e| AppError::Internal(format!("cannot read hosts file: {}", e)))?;
            let mut new_lines: Vec<String> = Vec::new();
            let mut removed = 0;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains(target) && !trimmed.starts_with('#') {
                    removed += 1;
                    continue;
                }
                new_lines.push(line.to_string());
            }
            if removed == 0 {
                return Ok(CommandExecutionResult {
                    command_id: "net.hosts-edit".into(),
                    title: "Hosts Editor".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: format!("No entry found matching '{}'", target),
                });
            }
            std::fs::write(&path, new_lines.join("\n"))
                .map_err(|e| AppError::Internal(format!("cannot write hosts file: {}", e)))?;
            Ok(CommandExecutionResult {
                command_id: "net.hosts-edit".into(),
                title: "Hosts Entry Removed".into(),
                output: format!("Removed {} entry(ies) matching '{}' from {}", removed, target, path),
                status: CommandExecutionStatus::Success,
                summary: format!("Removed {} entry(ies)", removed),
            })
        }
        "toggle" | "comment" | "uncomment" => {
            if parts.len() < 2 {
                return Ok(CommandExecutionResult {
                    command_id: "net.hosts-edit".into(),
                    title: "Hosts Editor".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: "Usage: toggle <hostname>".into(),
                });
            }
            let target = parts[1];
            let content = std::fs::read_to_string(&path)
                .map_err(|e| AppError::Internal(format!("cannot read hosts file: {}", e)))?;
            let mut new_lines: Vec<String> = Vec::new();
            let mut toggled = 0;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains(target) && !trimmed.trim_start().starts_with('#') {
                    new_lines.push(format!("#{}", line));
                    toggled += 1;
                } else if trimmed.trim_start().starts_with('#') {
                    let uncommented = line.trim_start().trim_start_matches('#').trim().to_string();
                    if uncommented.contains(target) {
                        new_lines.push(uncommented);
                        toggled += 1;
                    } else {
                        new_lines.push(line.to_string());
                    }
                } else {
                    new_lines.push(line.to_string());
                }
            }
            if toggled == 0 {
                return Ok(CommandExecutionResult {
                    command_id: "net.hosts-edit".into(),
                    title: "Hosts Editor".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: format!("No entry found matching '{}'", target),
                });
            }
            std::fs::write(&path, new_lines.join("\n"))
                .map_err(|e| AppError::Internal(format!("cannot write hosts file: {}", e)))?;
            Ok(CommandExecutionResult {
                command_id: "net.hosts-edit".into(),
                title: "Hosts Entry Toggled".into(),
                output: format!("Toggled {} entry(ies) matching '{}'", toggled, target),
                status: CommandExecutionStatus::Success,
                summary: format!("Toggled {} entry(ies)", toggled),
            })
        }
        _ => {
            Ok(CommandExecutionResult {
                command_id: "net.hosts-edit".into(),
                title: "Hosts Editor".into(),
                output: "Available commands:\n  list                - Show hosts file\n  add <IP> <host>     - Add entry\n  remove <host>       - Remove entry\n  toggle <host>       - Comment/uncomment entry".into(),
                status: CommandExecutionStatus::Info,
                summary: "Use list, add, remove, or toggle".into(),
            })
        }
    }
}

// ============================================================
// 015 - Reverse Proxy Tunnel Configuration Manager
// ============================================================
pub fn tunnel_manager(input: &str) -> Result<CommandExecutionResult, AppError> {
    let input = input.trim().to_lowercase();
    let parts: Vec<&str> = input.split_whitespace().collect();
    let action = if parts.is_empty() { "" } else { parts[0] };

    match action {
        "" | "status" => {
            let mut output = String::from("Tunnel Tool Status:\n\n");

            let tools = vec![
                ("ngrok", "ngrok"),
                ("localtunnel", "lt"),
                ("cloudflared", "cloudflared"),
                ("bore", "bore"),
                ("ssh", "ssh"),
            ];

            for (name, binary) in &tools {
                let installed = Command::new("where")
                    .args(&[binary])
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            String::from_utf8(o.stdout).ok().map(|s| s.lines().next().unwrap_or("").to_string())
                        } else {
                            None
                        }
                    });

                let running: Option<String> = Command::new("tasklist")
                    .args(&["/FI", &format!("IMAGENAME eq {}", binary), "/NH"])
                    .output()
                    .ok()
                    .and_then(|o| {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        if stdout.contains(binary) { Some("Running".into()) } else { None }
                    });

                match (installed, running) {
                    (Some(path), Some(_)) => {
                        output.push_str(&format!("  {}     INSTALLED at: {}  [ACTIVE]\n", name, path));
                    }
                    (Some(path), None) => {
                        output.push_str(&format!("  {}     INSTALLED at: {}  [idle]\n", name, path));
                    }
                    (None, _) => {
                        output.push_str(&format!("  {}     NOT INSTALLED\n", name));
                    }
                }
            }

            output.push_str("\nUse 'start <tool>' or 'stop <tool>' to manage tunnels.");

            Ok(CommandExecutionResult {
                command_id: "net.tunnel-mgr".into(),
                title: "Tunnel Manager".into(),
                output,
                status: CommandExecutionStatus::Success,
                summary: format!("{} tunnel tool(s) available", tools.iter().filter(|(_, b)| {
                    Command::new("where").args(&[b]).output().ok().map(|o| o.status.success()).unwrap_or(false)
                }).count()),
            })
        }
        "start" | "launch" => {
            if parts.len() < 2 {
                return Ok(CommandExecutionResult {
                    command_id: "net.tunnel-mgr".into(),
                    title: "Tunnel Manager".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: "Specify a tool to start: ngrok, localtunnel, cloudflared".into(),
                });
            }
            let tool = parts[1];
            let port = parts.get(2).unwrap_or(&"8080");

            match tool {
                "ngrok" => {
                    let url = format!("http://{}", port);
                    let _ = Command::new("cmd")
                        .args(&["/C", "start", "ngrok", "http", &url])
                        .spawn();
                    Ok(CommandExecutionResult {
                        command_id: "net.tunnel-mgr".into(),
                        title: "Tunnel Started".into(),
                        output: format!("Launched ngrok tunnel to {}.\nDashboard: http://127.0.0.1:4040", url),
                        status: CommandExecutionStatus::Success,
                        summary: "ngrok tunnel launching...".into(),
                    })
                }
                "localtunnel" | "lt" => {
                    let _ = Command::new("cmd")
                        .args(&["/C", "start", "npx", "lt", "--port", port])
                        .spawn();
                    Ok(CommandExecutionResult {
                        command_id: "net.tunnel-mgr".into(),
                        title: "Tunnel Started".into(),
                        output: format!("Launched localtunnel tunnel to port {}.", port),
                        status: CommandExecutionStatus::Success,
                        summary: "localtunnel tunnel launching...".into(),
                    })
                }
                "cloudflared" => {
                    let _ = Command::new("cmd")
                        .args(&["/C", "start", "cloudflared", "tunnel", "--url", &format!("http://localhost:{}", port)])
                        .spawn();
                    Ok(CommandExecutionResult {
                        command_id: "net.tunnel-mgr".into(),
                        title: "Tunnel Started".into(),
                        output: format!("Launched cloudflared tunnel to port {}.", port),
                        status: CommandExecutionStatus::Success,
                        summary: "cloudflared tunnel launching...".into(),
                    })
                }
                _ => Ok(CommandExecutionResult {
                    command_id: "net.tunnel-mgr".into(),
                    title: "Tunnel Manager".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: format!("Unknown tool '{}'. Supported: ngrok, localtunnel, cloudflared", tool),
                }),
            }
        }
        "stop" | "kill" => {
            if parts.len() < 2 {
                return Ok(CommandExecutionResult {
                    command_id: "net.tunnel-mgr".into(),
                    title: "Tunnel Manager".into(),
                    output: String::new(),
                    status: CommandExecutionStatus::Info,
                    summary: "Specify a tool to stop: ngrok, localtunnel, cloudflared".into(),
                });
            }
            let tool = match parts[1] {
                "ngrok" => "ngrok.exe",
                "localtunnel" | "lt" => "node.exe",
                "cloudflared" => "cloudflared.exe",
                other => other,
            };
            let _ = run_cmd("taskkill", &["/F", "/IM", tool]);
            Ok(CommandExecutionResult {
                command_id: "net.tunnel-mgr".into(),
                title: "Tunnel Stopped".into(),
                output: format!("Sent kill signal to {} processes.", tool),
                status: CommandExecutionStatus::Success,
                summary: format!("Stopped {}", tool),
            })
        }
        _ => {
            Ok(CommandExecutionResult {
                command_id: "net.tunnel-mgr".into(),
                title: "Tunnel Manager".into(),
                output: "Commands:\n  status              - Show tunnel tool status\n  start <tool> [port] - Start a tunnel\n  stop <tool>         - Stop a tunnel\n\nSupported tools: ngrok, localtunnel, cloudflared".into(),
                status: CommandExecutionStatus::Info,
                summary: "Use status, start, or stop".into(),
            })
        }
    }
}

// ============================================================
// 016 - Network Interception Curl Builder
// ============================================================
pub fn curl_builder(input: &str) -> Result<CommandExecutionResult, AppError> {
    if input.trim().is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "net.curl-builder".into(),
            title: "Curl Builder".into(),
            output: "Paste raw HTTP request/response log data to convert to a curl command.\n\nThe builder parses common log formats including:\n- Apache/Nginx access logs\n- Raw HTTP request dumps\n- cURL-style verbose output\n- Wireshark/tshark exports".into(),
            status: CommandExecutionStatus::Info,
            summary: "Paste raw network log data to convert to curl".into(),
        });
    }

    let mut method = String::from("GET");
    let mut url = String::new();
    let mut headers: Vec<String> = Vec::new();
    let mut body = String::new();
    let mut in_body = false;

    for line in input.lines() {
        let trimmed = line.trim();

        if trimmed.to_uppercase().starts_with("GET ")
            || trimmed.to_uppercase().starts_with("POST ")
            || trimmed.to_uppercase().starts_with("PUT ")
            || trimmed.to_uppercase().starts_with("PATCH ")
            || trimmed.to_uppercase().starts_with("DELETE ")
            || trimmed.to_uppercase().starts_with("HEAD ")
            || trimmed.to_uppercase().starts_with("OPTIONS ")
        {
            let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                method = parts[0].to_uppercase();
                let path = parts[1];
                if parts.len() >= 3 {
                    // Has HTTP version, extract host from headers or use path as-is
                    url = path.to_string();
                } else {
                    url = path.to_string();
                }
            }
            continue;
        }

        if trimmed.to_uppercase().starts_with("HOST: ") || trimmed.to_uppercase().starts_with("HOST:") {
            let host = trimmed.trim_start_matches(|c: char| c.is_alphabetic() || c == ':').trim();
            if !url.starts_with("http") {
                url = format!("https://{}{}", host, if url.starts_with('/') { url.clone() } else { format!("/{}", url) });
            }
            continue;
        }

        if trimmed.to_uppercase().starts_with("> ") || trimmed.to_uppercase().starts_with("< ") {
            continue;
        }

        if trimmed.is_empty() {
            in_body = true;
            continue;
        }

        if !in_body && trimmed.contains(':') && !trimmed.starts_with('[') {
            if let Some(idx) = trimmed.find(':') {
                let key = trimmed[..idx].trim();
                let value = trimmed[idx + 1..].trim();
                if !key.is_empty() && !value.is_empty() {
                    headers.push(format!("-H '{}: {}'", key, value));
                    continue;
                }
            }
        }

        if in_body {
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(trimmed);
        }
    }

    let mut curl = format!("curl -X {}", method);

    for h in &headers {
        curl.push(' ');
        curl.push_str(h);
    }

    if !body.is_empty() {
        let escaped = body.replace('\'', "'\\''");
        curl.push_str(&format!(" -d '{}'", escaped));
        if method == "GET" {
            curl = curl.replace("-X GET", "-X GET");
        }
    }

    if !url.is_empty() {
        if !url.starts_with("http") {
            curl.push_str(&format!(" 'http://localhost{}'", url));
        } else {
            curl.push_str(&format!(" '{}'", url));
        }
    } else {
        curl.push_str(" '<url>'");
    }

    Ok(CommandExecutionResult {
        command_id: "net.curl-builder".into(),
        title: "Curl Command".into(),
        output: curl,
        status: CommandExecutionStatus::Success,
        summary: "Raw log converted to curl command".into(),
    })
}

// ============================================================
// 017 - ICMP Ping / Traceroute Analyzer
// ============================================================
pub fn run_ping(input: &str) -> Result<CommandExecutionResult, AppError> {
    let target = input.trim();
    if target.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "net.ping".into(),
            title: "Ping".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Enter a hostname or IP address to ping (e.g. google.com or 8.8.8.8)".into(),
        });
    }

    let raw = run_cmd("ping", &["-n", "4", target])?;

    let mut output = String::new();
    output.push_str(&format!("Ping results for {}:\n\n", target));

    let sent = 4;
    let mut received = 0;
    let mut times: Vec<f64> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        output.push_str(trimmed);
        output.push('\n');

        if trimmed.contains("time=") || trimmed.contains("time<") {
            received += 1;
            if let Some(ms_str) = trimmed.split("time=").nth(1).or_else(|| trimmed.split("time<").nth(1)) {
                let ms_str = ms_str.trim_matches(|c: char| c.is_whitespace() || c == 'm' || c == 's');
                if let Ok(ms) = ms_str.parse::<f64>() {
                    times.push(ms);
                }
            }
        }
        if trimmed.contains("Reply from") {
            received += 1;
        }
    }

    let packet_loss = if sent > 0 {
        ((sent - received) as f64 / sent as f64) * 100.0
    } else {
        0.0
    };

    output.push_str(&format!(
        "\n--- Ping Statistics for {} ---\n", target
    ));
    output.push_str(&format!(
        "  Packets: Sent = {}, Received = {}, Lost = {} ({}% loss)\n",
        sent, received, sent - received, packet_loss as u32
    ));

    if !times.is_empty() {
        let min = times.iter().cloned().fold(f64::MAX, f64::min);
        let max = times.iter().cloned().fold(f64::MIN, f64::max);
        let avg = times.iter().sum::<f64>() / times.len() as f64;
        output.push_str(&format!(
            "  Round Trip: Min = {}ms, Max = {}ms, Avg = {:.1}ms\n",
            min as u32, max as u32, avg
        ));
    }

    let status = if received > 0 {
        CommandExecutionStatus::Success
    } else {
        CommandExecutionStatus::Error
    };

    let summary = format!("{} packets transmitted, {} received, {}% loss", sent, received, packet_loss as u32);

    Ok(CommandExecutionResult {
        command_id: "net.ping".into(),
        title: format!("Ping {}", target),
        output,
        status,
        summary,
    })
}

pub fn run_traceroute(input: &str) -> Result<CommandExecutionResult, AppError> {
    let target = input.trim();
    if target.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "net.trace".into(),
            title: "Traceroute".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Enter a hostname or IP address to traceroute (e.g. google.com)".into(),
        });
    }

    let raw = run_cmd("tracert", &["-d", target])?;

    let mut output = String::new();
    output.push_str(&format!("Traceroute to {}:\n\n", target));

    let mut hops = 0;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        output.push_str(trimmed);
        output.push('\n');
        if trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            hops += 1;
        }
    }

    Ok(CommandExecutionResult {
        command_id: "net.trace".into(),
        title: format!("Traceroute {}", target),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} hop(s) to {}", hops, target),
    })
}

// ============================================================
// 018 - Public/Private IP Discovery & Geolocation Check
// ============================================================
pub fn discover_ip(input: &str) -> Result<CommandExecutionResult, AppError> {
    let _ = input;

    let raw = run_cmd("ipconfig", &["/all"])?;

    let mut output = String::new();
    output.push_str("Network Configuration:\n\n");

    let mut current_adapter;
    let mut in_adapter = false;

    for line in raw.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Ethernet adapter") || trimmed.starts_with("Wireless LAN adapter") || trimmed.starts_with("Unknown adapter") {
            if in_adapter {
                output.push('\n');
            }
            current_adapter = trimmed.trim_end_matches(':').to_string();
            in_adapter = true;
            output.push_str(&format!("=== {} ===\n", current_adapter));
            continue;
        }

        if in_adapter {
            let lower = trimmed.to_lowercase();
            if lower.contains("ipv4 address") || lower.contains("ip address") {
                if let Some(ip) = trimmed.split(':').nth(1) {
                    output.push_str(&format!("  IP Address:       {}\n", ip.trim()));
                }
            } else if lower.contains("subnet mask") {
                if let Some(mask) = trimmed.split(':').nth(1) {
                    output.push_str(&format!("  Subnet Mask:      {}\n", mask.trim()));
                }
            } else if lower.contains("default gateway") {
                if let Some(gw) = trimmed.split(':').nth(1) {
                    let gw = gw.trim();
                    if !gw.is_empty() {
                        output.push_str(&format!("  Default Gateway:  {}\n", gw));
                    }
                }
            } else if lower.contains("dns server") {
                if let Some(dns) = trimmed.split(':').nth(1) {
                    output.push_str(&format!("  DNS Server:       {}\n", dns.trim()));
                }
            } else if lower.contains("physical address") || lower.contains("mac address") {
                if let Some(mac) = trimmed.split(':').nth(1) {
                    output.push_str(&format!("  MAC Address:      {}\n", mac.trim()));
                }
            } else if lower.contains("dhcp enabled") {
                if let Some(dhcp) = trimmed.split(':').nth(1) {
                    output.push_str(&format!("  DHCP Enabled:     {}\n", dhcp.trim()));
                }
            }
        }
    }

    // Try to get external IP
    if let Ok(ext_ip) = run_powershell(
        "(Invoke-WebRequest -Uri 'https://api.ipify.org' -TimeoutSec 5 -UseBasicParsing).Content.Trim()",
    ) {
        if !ext_ip.is_empty() && !ext_ip.to_lowercase().contains("invoke-webrequest") {
            output.push_str(&format!("\n=== External IP ===\n  Public IP:        {}\n", ext_ip));
        }
    }

    // Router gateway
    if let Ok(gateway) = run_powershell(
        "(Get-WmiObject Win32_IP4RouteTable | Where-Object { $_.Destination -eq '0.0.0.0' }).NextHop",
    ) {
        if !gateway.is_empty() {
            let gw = gateway.lines().next().unwrap_or("").trim();
            if !gw.is_empty() {
                output.push_str(&format!("\n=== Router ===\n  Gateway:          {}\n", gw));
            }
        }
    }

    // Hostname
    if let Ok(hostname) = std::env::var("COMPUTERNAME") {
        output.insert_str(0, &format!("Hostname: {}\n\n", hostname));
    }

    Ok(CommandExecutionResult {
        command_id: "net.ip-discover".into(),
        title: "IP Discovery".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "Local network configuration discovered".into(),
    })
}

// ============================================================
// 019 - Domain Name & SSL Expiry Validator
// ============================================================
pub fn check_domain(input: &str) -> Result<CommandExecutionResult, AppError> {
    let domain = input.trim();
    if domain.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "net.domain-check".into(),
            title: "Domain Check".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Enter a domain name (e.g. example.com) to check DNS and SSL certificate.".into(),
        });
    }

    let mut output = String::new();
    output.push_str(&format!("Domain: {}\n\n", domain));

    // DNS resolution via nslookup
    output.push_str("--- DNS Resolution ---\n");
    if let Ok(dns) = run_cmd("nslookup", &[domain]) {
        for line in dns.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(">") || trimmed.starts_with("Default") {
                continue;
            }
            if trimmed.contains("Address:") || trimmed.contains("Name:") || trimmed.contains("Aliases:") {
                output.push_str(&format!("  {}\n", trimmed));
            }
        }
    } else {
        output.push_str("  [DNS lookup failed]\n");
    }

    output.push('\n');

    // SSL certificate check via PowerShell
    output.push_str("--- SSL Certificate ---\n");
    let ps_script = format!(
        "$d='{}'; \
         $tcp=New-Object System.Net.Sockets.TcpClient; \
         try {{ \
           $tcp.ConnectAsync($d,443).Wait(5000)|Out-Null; \
           if(!$tcp.Connected){{Write-Output '[Connection timed out]';return}}; \
           $s=$tcp.GetStream(); \
           $ssl=New-Object System.Net.Security.SslStream($s,$false,{{$true}}); \
           $ssl.AuthenticateAsClient($d); \
           $cert=$ssl.RemoteCertificate; \
           Write-Output ('Subject: '+$cert.Subject); \
           Write-Output ('Issuer: '+$cert.Issuer); \
           Write-Output ('Effective: '+$cert.GetEffectiveDateString()); \
           Write-Output ('Expires: '+$cert.GetExpirationDateString()); \
           $exp=[datetime]$cert.GetExpirationDateString(); \
           $days=($exp-([datetime]::UtcNow)).Days; \
           Write-Output ('Days Remaining: '+$days); \
           Write-Output ('Serial: '+$cert.GetSerialNumberString()); \
           $ssl.Close(); \
           $tcp.Close() \
         }} catch {{Write-Output ('Error: '+$_.Exception.Message)}}",
        domain
    );

    if let Ok(cert_info) = run_powershell(&ps_script) {
        if cert_info.contains("Subject:") || cert_info.contains("[Connection timed out]") {
            for line in cert_info.lines() {
                output.push_str(&format!("  {}\n", line));
            }
        } else {
            output.push_str("  [No SSL certificate information available]\n");
            if !cert_info.is_empty() {
                output.push_str(&format!("  Raw: {}\n", cert_info));
            }
        }
    } else {
        output.push_str("  [SSL check failed - PowerShell required]\n");
    }

    Ok(CommandExecutionResult {
        command_id: "net.domain-check".into(),
        title: format!("Domain Check: {}", domain),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("DNS and SSL check complete for {}", domain),
    })
}

// ============================================================
// 020 - Local Subnet Sweep Engine
// ============================================================
pub fn sweep_subnet(input: &str) -> Result<CommandExecutionResult, AppError> {
    let input = input.trim();

    let base_ip = if input.is_empty() {
        // Auto-detect subnet from ipconfig
        let ipconfig = run_cmd("ipconfig", &["/all"]).unwrap_or_default();
        let mut detected = String::new();
        for line in ipconfig.lines() {
            let lower = line.trim().to_lowercase();
            if (lower.contains("ipv4 address") || lower.contains("ip address"))
                && !lower.contains("192.168.56")
                && !lower.contains("169.254")
            {
                if let Some(ip) = line.trim().split(':').nth(1) {
                    let ip = ip.trim();
                    let parts: Vec<&str> = ip.split('.').collect();
                    if parts.len() >= 3 {
                        detected = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
                    }
                }
            }
        }
        if detected.is_empty() {
            detected = "192.168.1".into();
        }
        detected
    } else {
        // Parse CIDR or IP prefix
        let clean = input.split('/').next().unwrap_or(input);
        let parts: Vec<&str> = clean.split('.').collect();
        if parts.len() >= 3 {
            format!("{}.{}.{}", parts[0], parts[1], parts[2])
        } else {
            return Ok(CommandExecutionResult {
                command_id: "net.subnet-sweep".into(),
                title: "Subnet Sweep".into(),
                output: String::new(),
                status: CommandExecutionStatus::Info,
                summary: "Enter a subnet prefix (e.g. 192.168.1 or 10.0.0.0/24)".into(),
            });
        }
    };

    let mut output = String::new();
    output.push_str(&format!("Subnet Sweep: {}.0/24\n\n", base_ip));
    output.push_str(&format!("{:<18} {:<20}\n", "IP Address", "Status"));
    output.push_str(&"-".repeat(40));
    output.push('\n');

    let mut active_hosts: Vec<(u32, String)> = Vec::new();

    // Ping hosts 1-254 with short timeout, using threads for speed
    let mut handles = Vec::new();
    let base = base_ip.clone();

    for i in 1..255 {
        let ip = format!("{}.{}", base, i);
        handles.push(std::thread::spawn(move || {
            let result = Command::new("ping")
                .args(&["-n", "1", "-w", "300", &ip])
                .output();
            match result {
                Ok(out) if out.status.success() => Some((i, ip)),
                _ => None,
            }
        }));
    }

    for handle in handles {
        if let Ok(Some((idx, ip))) = handle.join() {
            active_hosts.push((idx, ip));
        }
    }

    active_hosts.sort_by_key(|(idx, _)| *idx);

    let mut host_count = 0;
    for (_idx, ip) in &active_hosts {
        output.push_str(&format!("{:<18} [ACTIVE]\n", ip));
        host_count += 1;

        // Try to get hostname via DNS resolution
        if let Ok(dns) = run_cmd("nslookup", &[ip]) {
            for line in dns.lines() {
                let trimmed = line.trim();
                if trimmed.to_lowercase().contains("name:") {
                    if let Some(name) = trimmed.split(':').nth(1) {
                        output.push_str(&format!("  {:<18} {}\n", "", name.trim()));
                    }
                }
            }
        }
    }

    output.push_str(&format!(
        "\nScan complete. {} active host(s) found in {}.0/24",
        host_count, base_ip
    ));

    let summary = format!("Found {} active host(s) in subnet", host_count);

    Ok(CommandExecutionResult {
        command_id: "net.subnet-sweep".into(),
        title: "Subnet Sweep".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary,
    })
}
