use typelude_tooling_core::{RenderMode, RenderedText};

use crate::{caps::normalize_capability_name, naming::compress_symbol_name};

#[derive(Debug, Default)]
pub struct TypeludeRenderer;

impl TypeludeRenderer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn render_type_expression(&self, input: &str, mode: RenderMode) -> RenderedText {
        let text = render_expression(input.trim());
        RenderedText {
            mode,
            text,
        }
    }
}

fn render_expression(input: &str) -> String {
    if let Some(rendered) = render_array(input) {
        return rendered;
    }
    if let Some(rendered) = render_eapp(input) {
        return rendered;
    }
    if let Some(stripped) = input.strip_prefix('U') {
        if stripped.chars().all(|character| character.is_ascii_digit()) {
            return stripped.to_owned();
        }
    }
    if input.ends_with("Helper") || input.contains("::") {
        return compress_symbol_name(input);
    }
    normalize_capability_name(input)
}

fn render_array(input: &str) -> Option<String> {
    if input == "Nil" {
        return Some(String::from("[]"));
    }
    if !input.starts_with("Array<") || !input.ends_with('>') {
        return None;
    }

    let mut items = Vec::new();
    let mut current = input;
    loop {
        if current == "Nil" {
            break;
        }
        let inner = current.strip_prefix("Array<")?.strip_suffix('>')?;
        let parts = split_top_level(inner);
        if parts.len() != 2 {
            return None;
        }
        items.push(render_expression(parts[0].trim()));
        current = parts[1].trim();
    }

    Some(format!("[{}]", items.join(", ")))
}

fn render_eapp(input: &str) -> Option<String> {
    let mut args = Vec::new();
    let mut current = input.trim();

    while current.starts_with("EApp<") && current.ends_with('>') {
        let inner = current.strip_prefix("EApp<")?.strip_suffix('>')?;
        let parts = split_top_level(inner);
        if parts.len() != 2 {
            return None;
        }
        args.push(render_expression(parts[1].trim()));
        current = parts[0].trim();
    }

    if args.is_empty() {
        return None;
    }

    args.reverse();
    Some(format!("{}({})", compress_symbol_name(current), args.join(", ")))
}

fn split_top_level(input: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0_usize;

    for (index, character) in input.char_indices() {
        match character {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                result.push(input[start..index].trim());
                start = index + 1;
            },
            _ => {},
        }
    }
    result.push(input[start..].trim());
    result
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::RenderMode;

    use super::TypeludeRenderer;

    #[test]
    fn renders_array_as_list() {
        let rendered = TypeludeRenderer::new()
            .render_type_expression("Array<U1, Array<U2, Nil>>", RenderMode::Text);
        assert_eq!(rendered.text, "[1, 2]");
    }

    #[test]
    fn renders_nested_eapp() {
        let rendered = TypeludeRenderer::new()
            .render_type_expression("EApp<EApp<EIf, U1>, U0>", RenderMode::Text);
        assert_eq!(rendered.text, "If(1, 0)");
    }
}
