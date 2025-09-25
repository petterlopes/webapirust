pub const MAX_JSON_BODY_BYTES: usize = 16 * 1024; // 16 KiB

pub fn sanitize_for_logging(value: &str) -> String {
    let mut sanitized = value
        .chars()
        .filter(|c| !c.is_control() || *c == '\u{0009}' || *c == '\u{0020}')
        .collect::<String>();
    if sanitized.len() > 256 {
        sanitized.truncate(256);
    }
    sanitized.trim().to_owned()
}
