use regex::Regex;
use std::sync::LazyLock;

// Regex para limpeza e validação
pub static RE_MULTISPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());
pub static RE_NON_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\D").unwrap());
pub static RE_CHAVE_44: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{44})$").unwrap());
