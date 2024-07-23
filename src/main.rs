use std::process::Command;
use std::str;

use zap::BANNER;

#[derive(Debug)]
struct Output {
    stdout: String,
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        println!("\x1b[33m{}\x1b[0m", BANNER);
        println!("\x1b[33mUsage: zap <PORT>\x1b[0m");
        return;
    }

    let mut port = args[1].to_string();
    if !port.starts_with(":") {
        port = format!(":{}", port);
    }

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
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                parts[1].parse::<u32>().ok().map(|pid| pid.to_string())
            } else {
                None
            }
        })
        .collect();

    for pid in &pids {
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

    println!(
        "\x1b[33mZapped processes on port {} \x1b[0m\u{26A1}",
        port.split(":").last().unwrap()
    );
}
