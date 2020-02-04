use nix::unistd::getuid;
use std::io::{BufRead, BufReader, Read, Write};

fn write_message<S>(msg: &str, stream: &mut S) -> std::io::Result<()>
where
    S: Write,
{
    let mut buf = Vec::new();
    buf.extend(msg.bytes());
    buf.push(b'\r');
    buf.push(b'\n');
    stream.write_all(&buf)?;
    Ok(())
}

fn get_uid_as_hex() -> String {
    let uid = getuid();
    let mut tmp = uid.as_raw();
    let mut numbers = Vec::new();
    if tmp == 0 {
        return "30".to_owned();
    }
    while tmp > 0 {
        numbers.push(tmp % 10);
        tmp /= 10;
    }
    let mut hex = String::new();
    for idx in 0..numbers.len() {
        hex.push_str(match numbers[numbers.len() - 1 - idx] {
            0 => "30",
            1 => "31",
            2 => "32",
            3 => "33",
            4 => "34",
            5 => "35",
            6 => "36",
            7 => "37",
            8 => "38",
            9 => "39",
            _ => unreachable!(),
        })
    }

    hex
}

pub enum AuthResult {
    Ok,
    Rejected,
}

pub fn do_auth<S>(stream: &mut S) -> std::io::Result<AuthResult>
where
    S: Write + Read,
{
    // send a null byte as the first thing
    stream.write_all(&[0])?;
    write_message(&format!("AUTH EXTERNAL {}", get_uid_as_hex()), stream)?;

    let mut msg = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut msg)?;

    if msg.starts_with("OK") {
        Ok(AuthResult::Ok)
    } else {
        Ok(AuthResult::Rejected)
    }
}

pub fn negotiate_unix_fds<S>(stream: &mut S) -> std::io::Result<AuthResult>
where
    S: Write + Read,
{
    write_message("NEGOTIATE_UNIX_FD", stream)?;

    let mut msg = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut msg)?;

    if msg.starts_with("AGREE_UNIX_FD") {
        Ok(AuthResult::Ok)
    } else {
        Ok(AuthResult::Rejected)
    }
}

pub fn send_begin<S>(stream: &mut S) -> std::io::Result<()>
where
    S: Write,
{
    write_message("BEGIN", stream)?;
    Ok(())
}
