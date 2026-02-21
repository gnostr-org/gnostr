pub static RELAYS_YAML: &[u8] = include_bytes!("../../../../config/relays.yaml");

pub fn get_relays() -> Vec<String> {
    let content = std::str::from_utf8(RELAYS_YAML).unwrap_or_default();
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}
