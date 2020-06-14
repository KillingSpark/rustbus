//! Deals with authentication to the other side. You probably do not need this.

use nix::unistd::getuid;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

fn write_message(msg: &str, stream: &mut UnixStream) -> std::io::Result<()> {
    let mut buf = Vec::new();
    buf.extend(msg.bytes());
    buf.push(b'\r');
    buf.push(b'\n');
    stream.write_all(&buf)?;
    Ok(())
}

fn has_line_ending(buf: &[u8]) -> bool {
    for idx in 1..buf.len() {
        if buf[idx - 1] == b'\r' && buf[idx] == b'\n' {
            return true;
        }
    }
    false
}

fn find_line_ending(buf: &[u8]) -> Option<usize> {
    for idx in 1..buf.len() {
        if buf[idx - 1] == b'\r' && buf[idx] == b'\n' {
            return Some(idx - 1);
        }
    }
    None
}

fn read_message(stream: &mut UnixStream, buf: &mut Vec<u8>) -> std::io::Result<String> {
    let mut tmpbuf = [0u8; 512];
    while !has_line_ending(&buf) {
        let bytes = stream.read(&mut tmpbuf[..])?;
        buf.extend(&tmpbuf[..bytes])
    }
    let idx = find_line_ending(&buf).unwrap();
    let line = buf.drain(0..idx).collect::<Vec<_>>();
    Ok(String::from_utf8(line).unwrap())
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

pub fn do_auth(stream: &mut UnixStream) -> std::io::Result<AuthResult> {
    // send a null byte as the first thing
    stream.write_all(&[0])?;
    write_message(&format!("AUTH EXTERNAL {}", get_uid_as_hex()), stream)?;

    let mut read_buf = Vec::new();
    let msg = read_message(stream, &mut read_buf)?;
    if msg.starts_with("OK") {
        Ok(AuthResult::Ok)
    } else {
        Ok(AuthResult::Rejected)
    }
}

pub fn negotiate_unix_fds(stream: &mut UnixStream) -> std::io::Result<AuthResult> {
    write_message("NEGOTIATE_UNIX_FD", stream)?;

    let mut read_buf = Vec::new();
    let msg = read_message(stream, &mut read_buf)?;
    if msg.starts_with("AGREE_UNIX_FD") {
        Ok(AuthResult::Ok)
    } else {
        Ok(AuthResult::Rejected)
    }
}

pub fn send_begin(stream: &mut UnixStream) -> std::io::Result<()> {
    write_message("BEGIN", stream)?;
    Ok(())
}
