use std::io::{self, Write};
use std::process::Command;
use std::str;

use zap::BANNER;

#[derive(Debug)]
struct Output {
    stdout: String,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pid: String,
    owner: String,
    command: String,
}

fn parse_lsof_line(line: &str) -> Option<ProcessInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 3 {
        return None;
    }

    let command = parts[0];
    let pid = parts[1];
    let owner = parts[2];

    pid.parse::<u32>().ok().map(|_| ProcessInfo {
        pid: pid.to_string(),
        owner: owner.to_string(),
        command: command.to_string(),
    })
}

fn get_processes_for_port(port: &str) -> Vec<ProcessInfo> {
    let output = Command::new("lsof")
        .arg("-i")
        .arg(&port)
        .output()
        .expect("failed to execute process");

    let data = Output {
        stdout: String::from_utf8(output.stdout).unwrap(),
    };

    let processes: Vec<ProcessInfo> = data
        .stdout
        .lines()
        .skip(1)
        .filter_map(|line| parse_lsof_line(line))
        .collect();

    let mut deduped_processes = Vec::new();
    let mut seen_pids = std::collections::HashSet::new();

    for process in processes {
        if seen_pids.insert(process.pid.clone()) {
            deduped_processes.push(process);
        }
    }

    return deduped_processes;
}

fn get_current_user() -> String {
    Command::new("whoami")
        .output()
        .map(|output| {
            String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

fn prompt_user_confirmation(processes_needing_sudo: &[ProcessInfo]) -> bool {
    println!("\x1b[31m⚠️  WARNING: The following processes require sudo to kill:\x1b[0m");
    for process in processes_needing_sudo {
        println!(
            "  PID {} ({}): {} - owned by {}",
            process.pid, process.command, process.pid, process.owner
        );
    }
    println!();
    println!("\x1b[33m⚠️  Are you sure you wanna sudo? (y/N)\x1b[0m");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

pub struct Zap<'a> {
    pub processes: Vec<ProcessInfo>,
    pub port: &'a str,
}

impl<'a> Zap<'a> {
    pub fn new(port: &'a str) -> Self {
        let processes = get_processes_for_port(port);
        Self { processes, port }
    }

    fn kill_processes(self: Self) {
        let processes = self.processes;
        if processes.is_empty() {
            println!(
                "\x1b[33mNo processes found on port {} \x1b[0m",
                self.port.split(":").last().unwrap()
            );
            println!(
                "\x1b[33mNote: Run with 'sudo zap {}' to see processes from all users\x1b[0m",
                self.port.split(":").last().unwrap()
            );
            return;
        }

        let current_user = get_current_user();
        let mut owned_processes = Vec::new();
        let mut foreign_processes = Vec::new();

        for process in processes {
            if process.owner == current_user {
                owned_processes.push(process);
            } else {
                foreign_processes.push(process);
            }
        }

        let mut killed_count = 0;

        for process in &owned_processes {
            let kill_output = Command::new("kill")
                .arg("-9")
                .arg(&process.pid)
                .output()
                .expect("failed to execute kill command");

            if kill_output.stderr.is_empty() {
                killed_count += 1;
            } else {
                eprintln!(
                    "Failed to kill PID {}: {}",
                    process.pid,
                    str::from_utf8(&kill_output.stderr).unwrap().trim()
                );
            }
        }

        // Handle foreign processes if any exist
        if !foreign_processes.is_empty() {
            if prompt_user_confirmation(&foreign_processes) {
                for process in &foreign_processes {
                    let kill_output = Command::new("sudo")
                        .arg("kill")
                        .arg("-9")
                        .arg(&process.pid)
                        .output()
                        .expect("failed to execute sudo kill command");

                    if kill_output.stderr.is_empty() {
                        killed_count += 1;
                    } else {
                        eprintln!(
                            "Failed to sudo kill PID {}: {}",
                            process.pid,
                            str::from_utf8(&kill_output.stderr).unwrap().trim()
                        );
                    }
                }
            } else {
                println!(
                    "\x1b[33mSkipped {} processes requiring sudo\x1b[0m",
                    foreign_processes.len()
                );
            }
        }

        if killed_count > 0 {
            println!(
                "\x1b[33mZapped {} process(es) on port {} \x1b[0m\u{26A1}",
                killed_count,
                self.port.split(":").last().unwrap()
            );
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        println!("\x1b[33m{}\x1b[0m", BANNER);
        println!("\x1b[33mUsage: zap <PORT> <--node|--google>\x1b[0m");
        println!("\x1b[33mOptions:\x1b[0m");
        println!("\x1b[33m  -n, --node\x1b[0m");
        return;
    }

    let port = if args[1].starts_with(":") {
        args[1].clone()
    } else {
        format!(":{}", args[1])
    };

    let zap = Zap::new(&port);
    zap.kill_processes();
}
