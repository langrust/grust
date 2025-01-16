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

/// Transforms camel case strings into snake case.
///
/// Snake case strings are delimited by underscores `_` and are all lowercase. Camel case strings
/// are lowercase, but for every word the first letter is capitalized.
///
/// This function preserves consecutive digits.
///
/// ```
/// # compiler_common::prelude! { to_snake_case }
///
/// let string = to_snake_case("MyNode76");
/// let control = format!("my_node_76");
/// assert_eq!(string, control)
/// ```
pub fn to_snake_case(s: impl AsRef<str>) -> String {
    let s = s.as_ref();

    // Identify already-snake-case strings, the code below is such that if an alphabetic character
    // is directly followed by a digit (`my_ident2`) then an underscore will be inserted between
    // them (`my_ident_2`).
    //
    // So this function would not preserve already-snake-case strings, which is bad, hence if
    // there're no uppercase ascii letters we just return the `String`ification of `s`.
    if !s.chars().any(|c| c.is_ascii_uppercase()) {
        return s.into();
    }
    let mut res = String::with_capacity(s.len() * 2);
    enum Prev {
        Up,
        Low,
        Num,
    }
    use Prev::*;
    let mut prev = None;
    for char in s.chars() {
        if !char.is_ascii_alphanumeric() {
            res.push(char);
            prev = None;
        } else if char.is_ascii_lowercase() {
            res.push(char);
            prev = Some(Low);
        } else {
            // `char.is_numeric()` is seen as equivalent to `!char.is_uppercase()` because of the
            // checks of the previous branches
            let is_numeric = char.is_numeric();
            match (&prev, is_numeric) {
                // currently at the first character
                (None, is_num) => {
                    res.push(char.to_ascii_lowercase());
                    prev = Some(if is_num { Num } else { Up })
                }
                // no separation between two digits
                (Some(Num), true) => {
                    res.push(char);
                    // prev still Num
                }
                // cases where a `_` separator is needed
                // - any prev and current is Up
                (_, false) => {
                    res.push('_');
                    res.push(char.to_ascii_lowercase());
                    prev = Some(Up);
                }
                // - current is Num (prev can't be a Num, caught above)
                (_, true) => {
                    res.push('_');
                    res.push(char.to_ascii_lowercase());
                    prev = Some(Num);
                }
            }
        }
    }
    res.shrink_to_fit();
    res
}

#[cfg(test)]
mod to_snake_case {
    use super::to_snake_case;

    #[test]
    fn simple() {
        let string = to_snake_case("MyNodeO");
        let control = format!("my_node_o");
        assert_eq!(string, control)
    }

    #[test]
    fn consecutive_numerics() {
        let string = to_snake_case("With24Nums1");
        let control = format!("with_24_nums_1");
        assert_eq!(string, control)
    }

    #[test]
    fn consecutive_upper() {
        let string = to_snake_case("WithConsecutiveUPs");
        let control = format!("with_consecutive_u_ps");
        assert_eq!(string, control)
    }

    #[test]
    fn snake_no_change() {
        let data = vec![
            "some_ident",
            "nums_76523",
            "nums7410",
            "many____underscores___",
            "_more_underscores_",
            "_____too_many____underscores__",
        ];
        for s in data {
            let res = to_snake_case(s);
            assert_eq!(s, &res)
        }
    }
}
