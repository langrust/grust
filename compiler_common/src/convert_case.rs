/// Transforms snake case strings into camel case.
///
/// Snake case strings are delimited by underscores `_` and are all lowercase.
/// Camel case strings are lowercase, but for every word the first letter is capitalized.
///
/// ```
/// # compiler_common::prelude! { to_camel_case }
///
/// let string = to_camel_case("my_node_o");
/// let control = format!("MyNodeO");
/// assert_eq!(string, control)
/// ```
pub fn to_camel_case(s: impl AsRef<str>) -> String {
    s.as_ref()
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            if let Some(a) = chars.next() {
                a.to_uppercase().chain(chars.as_str().chars()).collect()
            } else {
                String::new()
            }
        })
        .collect()
}

#[cfg(test)]
mod to_camel_case {
    use super::to_camel_case;

    #[test]
    fn should_convert_snake_case_identifier_into_to_camel_case() {
        let string = to_camel_case("my_node_o");
        let control = format!("MyNodeO");
        assert_eq!(string, control)
    }

    #[test]
    fn should_convert_hybrid_identifier_into_to_camel_case() {
        let string = to_camel_case("my_node_oInput");
        let control = format!("MyNodeOInput");
        assert_eq!(string, control)
    }

    #[test]
    fn should_trim_snake_case_identifier() {
        let string = to_camel_case("my_node_o_");
        let control = format!("MyNodeO");
        assert_eq!(string, control)
    }
}
