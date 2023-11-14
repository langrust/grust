/// Transforms snake case strings into camel case.
/// 
/// Snake case strings are delimited by underscores `_` and are all lowercase.
/// Camel case strings are lowercase, but for every word the first letter is capitalized.
///
/// ```
/// use grustine::common::convert_case::camel_case;
/// 
/// let string = camel_case("my_node_o");
/// let control = format!("MyNodeO");
/// assert_eq!(string, control)
/// ```
pub fn camel_case(s: &str) -> String {
    s.split('_')
    .map(|word| {
        let mut chars = word.chars();
        if let Some(a) = chars.next() {
            a.to_uppercase()
                .chain(chars.as_str().chars())
                .collect()
        } else {
            String::new()
        }
    })
    .collect()
}

#[cfg(test)]
mod camel_case {
    use super::camel_case;

    #[test]
    fn should_convert_snake_case_identifier_into_camel_case() {
        let string = camel_case("my_node_o");
        let control = format!("MyNodeO");
        assert_eq!(string, control)
    }

    #[test]
    fn should_convert_hybrid_identifier_into_camel_case() {
        let string = camel_case("my_node_oInput");
        let control = format!("MyNodeOInput");
        assert_eq!(string, control)
    }

    #[test]
    fn should_trim_snake_case_identifier() {
        let string = camel_case("my_node_o_");
        let control = format!("MyNodeO");
        assert_eq!(string, control)
    }
}
