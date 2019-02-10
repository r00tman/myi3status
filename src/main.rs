use mpd::Client;
use serde_json::{json, Value};

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

struct MpdStatus {
    conn: Option<Client>,
}

impl MpdStatus {
    fn new() -> MpdStatus {
        MpdStatus {
            conn: None,
        }
    }

    fn get_or_connect(&mut self) -> Result<&mut Client, mpd::error::Error> {
        if self.conn.as_mut().map_or(true, |c| c.ping().is_err()) {
            match Client::connect("127.0.0.1:6600") {
                Ok(conn) => self.conn = Some(conn),
                Err(err) => return Err(err)
            }
        }
        Ok(self.conn.as_mut().unwrap())
    }

    fn get_text(&mut self) -> String {
        let conn = self.get_or_connect();
        if let Err(err) = conn {
            return format!("{}", err);
        }
        let conn = conn.unwrap();

        use mpd::status::State::*;
        match conn.currentsong() {
            Ok(Some(song)) => format!(
                "{} - {} ({})",
                song.tags
                    .get(&"Artist".to_owned())
                    .unwrap_or(&"no artist".to_owned()),
                song.title
                    .unwrap_or("no title".to_owned()),
                match conn.status() {
                    Ok(status) => match status.state {
                        Stop => "stopped",
                        Play => "playing",
                        Pause => "paused",
                    },
                    Err(_) => "unknown",
                }),
            Ok(None) => "no song".to_owned(),
            Err(err) => format!("{}", err)
        }
    }
}

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

    let mut mpd = MpdStatus::new();

    for line in br.lines() {
        if let Ok(mut line) = line {
            let is_cont = line.starts_with(',');
            let line = if is_cont {line.split_off(1)} else {line};
            let v: Result<Value, _> = serde_json::from_str(&line);

            if let Ok(Value::Array(mut vec)) = v {
                let mpd = json!({
                    "name": "mpd",
                    "instance": "local",
                    "markup": "none",
                    "full_text": "M: ".to_owned()+&mpd.get_text(),
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
