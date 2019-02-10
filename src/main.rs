use serde_json::{json, Value, Result};

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn main() {
    let cmd = Command::new("i3status")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Can't start i3status");

    let out = cmd.stdout
        .expect("Can't open child's stdout");

    let mut br = BufReader::new(out);

    for _ in 0..2 {
        let mut tmp = String::new();
        br.read_line(&mut tmp).expect("Can't read header lines");
        print!("{}", tmp);
    }

    for line in br.lines() {
        if let Ok(mut line) = line {
            let is_cont = line.starts_with(',');
            let line = if is_cont {line.split_off(1)} else {line};
            let v: Result<Value> = serde_json::from_str(&line);

            if let Ok(Value::Array(mut vec)) = v {
                let mpd = json!({
                    "name": "mpd",
                    "instance": "local",
                    "markup": "none",
                    "full_text": "testing stuff",
                });
                vec.insert(0, mpd);

                let cont = if is_cont {","} else {""};
                println!("{}{}", cont, Value::Array(vec));
            } else {
                println!("{}", line);
            }
        }
    }
}
