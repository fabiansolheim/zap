use std::process::Command;
use std::str;

#[derive(Debug)]
struct Output {
    stdout: String,
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() != 2 {
        println!("Usage: zap <PORT>");
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

    let node_pids: Vec<String> = data
        .stdout
        .lines()
        .filter(|line| line.contains("node"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                parts[1].parse::<u32>().ok().map(|pid| pid.to_string())
            } else {
                None
            }
        })
        .collect();

    for pid in &node_pids {
        let _ = Command::new("kill")
            .arg("-9")
            .arg(pid)
            .output()
            .expect("failed to execute kill command");

        /*  if !kill_output.stdout.is_empty() {
            println!()
        }
        if !kill_output.stderr.is_empty() {
            println!("stderr: {}", str::from_utf8(&kill_output.stderr).unwrap());
        } */
    }
    println!("\u{26A1}\x1b[33m Zapped node process! \x1b[0m");
}
