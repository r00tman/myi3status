use mpd::Client;
use time::Duration;
use serde_json::{json, Value};

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

struct MpdStatus {
    conn: Option<Client>,
}

fn format_duration(d: Duration) -> String {
    let mut carry = false;
    let mut res = String::new();
    if d.num_weeks() > 0 {
        res += &format!("{}w:", d.num_weeks());
        carry = true;
    }
    if carry || d.num_days() > 0 {
        res += &format!("{}d:", d.num_days()%7);
        carry = true;
    }
    if carry || d.num_hours() > 0 {
        res += &format!("{:02}:", d.num_hours()%24);
    }
    res += &format!("{:02}:", d.num_minutes()%60);
    res += &format!("{:02}", d.num_seconds()%60);
    res
}

fn format_time(prefix: &str, time: Option<(Duration, Duration)>) -> String {
    prefix.to_owned() + &match time {
        Some((elapsed, total)) =>
            format!(" {}/{}",
                    format_duration(elapsed),
                    format_duration(total)),
        None =>
            "".to_owned()
    }
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

    fn get_text(&mut self) -> Result<String, mpd::error::Error> {
        let conn = self.get_or_connect()?;

        use mpd::status::State::*;
        match conn.currentsong()? {
            Some(song) => Ok(format!(
                "{} - {} ({})",
                song.tags
                    .get(&"Artist".to_owned())
                    .unwrap_or(&"no artist".to_owned()),
                song.title
                    .unwrap_or_else(|| "no title".to_owned()),
                match conn.status() {
                    Ok(status) => format_time(
                        match status.state {
                            Stop => "stopped",
                            Play => "playing",
                            Pause => "paused",
                        }, status.time),
                    Err(_) => "unknown".to_owned(),
                })),
            None => Ok("no song".to_owned()),
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
                if let Ok(mpd_text) = mpd.get_text() {
                    let mpd = json!({
                        "name": "mpd",
                        "instance": "local",
                        "markup": "none",
                        "full_text": "M: ".to_owned()+&mpd_text,
                    });
                    vec.insert(0, mpd);
                }

                let cont = if is_cont {","} else {""};
                println!("{}{}", cont, Value::Array(vec));
            } else {
                println!("{}", line);
            }
        }
    }
}
