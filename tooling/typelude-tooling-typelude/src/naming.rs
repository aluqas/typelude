#[must_use]
pub fn compress_symbol_name(input: &str) -> String {
    let last_segment = input.rsplit("::").next().unwrap_or(input).trim_matches('`');
    match last_segment {
        "EApp" => String::from("Apply"),
        "EIf" => String::from("If"),
        "EWhile" => String::from("While"),
        "EMap" => String::from("Map"),
        "EGet" => String::from("Get"),
        symbol if symbol.ends_with("Helper") => symbol.trim_end_matches("Helper").to_owned(),
        symbol => symbol.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::compress_symbol_name;

    #[test]
    fn compresses_helper_names() {
        assert_eq!(compress_symbol_name("typelude_std::std::col::array::GetHelper"), "Get");
        assert_eq!(compress_symbol_name("EIf"), "If");
    }
}
