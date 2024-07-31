use core::fmt;
use std::process::Command;
use std::str;

use zap::BANNER;

#[derive(Debug)]
struct Output {
    stdout: String,
}

#[derive(Debug, PartialEq)]
enum Flag {
    Node,
    Google,
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Flag {
    fn from_str(flag: &str) -> Option<Self> {
        match flag {
            "-n" | "--node" => Some(Flag::Node),
            "-g" | "--google" => Some(Flag::Google),
            _ => None,
        }
    }
}
fn parse_line(line: &str, flags: &[Flag]) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let process_name = parts[0];
    let process_pid = parts[1];

    let is_node_process = flags.contains(&Flag::Node) && process_name == "node";
    let is_google_process = flags.contains(&Flag::Google) && process_name == "Google";

    if is_node_process || is_google_process {
        return process_pid.parse::<u32>().ok().map(|pid| pid.to_string());
    }

    if !flags.contains(&Flag::Node) && !flags.contains(&Flag::Google) {
        return process_pid.parse::<u32>().ok().map(|pid| pid.to_string());
    }

    None
}

fn get_processes_for_port(port: &str, flags: &[Flag]) -> Vec<String> {
    let output = Command::new("lsof")
        .arg("-i")
        .arg(&port)
        .output()
        .expect("failed to execute process");

    let data = Output {
        stdout: String::from_utf8(output.stdout).unwrap(),
    };

    let pids: Vec<String> = data
        .stdout
        .lines()
        .skip(1)
        .filter_map(|line| parse_line(line, flags))
        .collect();

    let mut deduped_pids: Vec<String> = pids.clone();
    deduped_pids.dedup();

    return deduped_pids;
}

fn kill_processes(pids: &[String]) {
    for pid in pids {
        let kill_output = Command::new("kill")
            .arg("-9")
            .arg(pid)
            .output()
            .expect("failed to execute kill command");

        if !kill_output.stdout.is_empty() {
            return;
        }
        if !kill_output.stderr.is_empty() {
            return println!("stderr: {}", str::from_utf8(&kill_output.stderr).unwrap());
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

    let flags: Vec<Flag> = args
        .iter()
        .skip(2)
        .filter_map(|arg| Flag::from_str(arg))
        .collect();

    let pids = get_processes_for_port(&port, &flags);
    if pids.is_empty() {
        println!("\x1b[33mNo processes found on port {}\x1b[0m", port);
        return;
    }

    kill_processes(&pids);

    println!(
        "\x1b[33mZapped processes on port {} \x1b[0m\u{26A1}",
        port.split(":").last().unwrap()
    );
}
