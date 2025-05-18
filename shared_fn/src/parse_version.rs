pub fn parse_versions(version: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = version
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    (parts[0], parts[1], parts[2])
}
