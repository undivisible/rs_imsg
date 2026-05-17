pub fn var(primary: &str, legacy: &str) -> Option<String> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(legacy).ok())
}

pub fn var_or(primary: &str, legacy: &str, default: &str) -> String {
    var(primary, legacy).unwrap_or_else(|| default.to_string())
}
