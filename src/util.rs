use anyhow::{Result, bail};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// 諸々の便利関数

pub fn normalize_device(letter: &str) -> Result<String> {
    let s = letter.trim();
    if s.is_empty() {
        bail!("empty drive letter");
    }
    if s.starts_with(r"\\.\") || s.starts_with(r"\\?\") {
        return Ok(s.to_string());
    }
    if s.len() == 1 && s.as_bytes()[0].is_ascii_alphabetic() {
        return Ok(format!(r"\\.\{}:", s.to_ascii_uppercase()));
    }
    if s.len() == 2 && s.ends_with(':') && s.as_bytes()[0].is_ascii_alphabetic() {
        return Ok(format!(r"\\.\{}", s.to_ascii_uppercase()));
    }
    bail!("unsupported drive letter: {}", s)
}

pub fn normalize_candidate_path(raw: &str) -> String {
    let mut p = raw.replace('/', "\\");
    if p.starts_with(r"\??\") {
        p = p.trim_start_matches(r"\??\").to_string();
    } else if p.starts_with(r"\\?\") {
        p = p.trim_start_matches(r"\\?\").to_string();
    } else if p.starts_with(r"\\.\") {
        p = p.trim_start_matches(r"\\.\").to_string();
    }
    if p.len() >= 3 && p.as_bytes()[1] == b':' && p.as_bytes()[2] == b'\\' {
        p = format!("\\{}", &p[3..]);
    }
    if p.len() >= 4
        && p.as_bytes()[0] == b'\\'
        && p.as_bytes()[2] == b':'
        && p.as_bytes()[3] == b'\\'
    {
        p = format!("\\{}", &p[4..]);
    }
    if !p.starts_with('\\') {
        p.insert(0, '\\');
    }
    while p.starts_with("\\\\") {
        p.remove(0);
    }
    p
}

pub fn normalize_and_canonicalize_for_key(raw: &str) -> String {
    let mut p = raw.replace('/', "\\");
    if p.starts_with(r"\??\") {
        p = p[r"\??\".len()..].to_string();
    } else if p.starts_with(r"\\?\") {
        p = p[r"\\?\".len()..].to_string();
    } else if p.starts_with(r"\\.\") {
        p = p[r"\\.\".len()..].to_string();
    }
    if p.len() >= 3 && p.as_bytes()[1] == b':' && p.as_bytes()[2] == b'\\' {
        p = format!(r"\{}", &p[3..]);
    }
    if p.len() >= 4
        && p.as_bytes()[0] == b'\\'
        && p.as_bytes()[2] == b':'
        && p.as_bytes()[3] == b'\\'
    {
        p = format!(r"\{}", &p[4..]);
    }
    if !p.starts_with('\\') {
        p.insert(0, '\\');
    }
    let mut stack: Vec<&str> = Vec::new();
    for part in p.split('\\') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if !stack.is_empty() {
                stack.pop();
            }
            continue;
        }
        stack.push(part);
    }
    if stack.is_empty() {
        "\\".to_string()
    } else {
        let mut out = String::from("\\");
        out.push_str(&stack.join("\\"));
        out.to_lowercase()
    }
}

pub fn unix_ts_to_system_time(ts: i64) -> SystemTime {
    if ts <= 0 {
        UNIX_EPOCH
    } else {
        UNIX_EPOCH + Duration::from_secs(ts as u64)
    }
}

pub fn humanize_bytes(mut b: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut idx = 0usize;
    while b >= 1024 && idx < UNITS.len() - 1 {
        b /= 1024;
        idx += 1;
    }
    format!("{} {}", b, UNITS[idx])
}
