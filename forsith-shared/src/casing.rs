#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Casing {
    Snake,
    UpperSnake,
    Kebab,
    Dromedary,
    Pascal,
}

pub fn change_casing(source: &str, casing: Casing) -> String {
    let mut dest = String::new();
    write_changed_casing(source, &mut dest, casing);
    dest
}

pub fn write_changed_casing(source: &str, dest: &mut String, casing: Casing) {
    let mut chars = source.chars().peekable();

    while let Some(c) = chars.peek() && !c.is_alphanumeric() {
        dest.push(*c);
        chars.next();
    }

    let mut rchars = chars.clone().rev().peekable();
    while let Some(c) = rchars.peek() && !c.is_alphanumeric() {rchars.next();}
    let mut casing_chars = chars.by_ref().take(rchars.count());

    match casing {
        Casing::Snake => {
            if let Some(c) = casing_chars.next() {dest.push(c.to_ascii_lowercase())}

            for c in casing_chars {
                if c.is_uppercase() {
                    dest.push('_');
                    dest.push(c.to_ascii_lowercase());
                }  else if c == '-' {
                    dest.push('_');
                } else {
                    dest.push(c);
                }
            }
        }
        Casing::UpperSnake => {
            if let Some(c) = casing_chars.next() {dest.push(c.to_ascii_uppercase())}

            for c in casing_chars {
                if c.is_uppercase() {
                    dest.push('_');
                    dest.push(c);
                } else if c == '-' {
                    dest.push('_');
                } else {
                    dest.push(c.to_ascii_uppercase());
                }
            }
        },
        Casing::Kebab => {
            if let Some(c) = casing_chars.next() {dest.push(c.to_ascii_lowercase())}

            for c in casing_chars {
                if c.is_uppercase() {
                    dest.push('-');
                    dest.push(c.to_ascii_lowercase());
                } else if c == '_' {
                    dest.push('-');
                } else {
                    dest.push(c);
                }
            }
        },
        Casing::Dromedary => {
            if let Some(c) = casing_chars.next() {dest.push(c.to_ascii_lowercase())}

            while let Some(c) = casing_chars.next() {
                if c == '_' || c == '-' {
                    if let Some(next_c) = casing_chars.next() && next_c.is_alphabetic() {
                        dest.push(next_c.to_ascii_uppercase());
                    }
                } else {
                    dest.push(c);
                }
            }
        },
        Casing::Pascal => {
            if let Some(c) = casing_chars.next() {dest.push(c.to_ascii_uppercase())}

            while let Some(c) = casing_chars.next() {
                if c == '_' || c == '-' {
                    if let Some(next_c) = casing_chars.next() && next_c.is_alphabetic() {
                        dest.push(next_c.to_ascii_uppercase());
                    }
                } else {
                    dest.push(c);
                }
            }
        }
    }

    chars.for_each(|c| dest.push(c));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_casing(source: &str, expected: &str, casing: Casing) {
        let mut dest = String::new();
        write_changed_casing(source, &mut dest, casing);
        assert_eq!(dest, expected);
    }

    #[test]
    fn test_snake_case() {
        test_casing("HelloWorld", "hello_world", Casing::Snake);
        test_casing("hello-world", "hello_world", Casing::Snake);
        test_casing("hello_world", "hello_world", Casing::Snake);
    }

    #[test]
    fn test_upper_snake_case() {
        test_casing("HelloWorld", "HELLO_WORLD", Casing::UpperSnake);
        test_casing("hello-world", "HELLO_WORLD", Casing::UpperSnake);
        test_casing("hello_world", "HELLO_WORLD", Casing::UpperSnake);
    }

    #[test]
    fn test_kebab_case() {
        test_casing("HelloWorld", "hello-world", Casing::Kebab);
        test_casing("hello_world", "hello-world", Casing::Kebab);
        test_casing("hello-world", "hello-world", Casing::Kebab);
    }

    #[test]
    fn test_dromedary_case() {
        test_casing("HelloWorld", "helloWorld", Casing::Dromedary);
        test_casing("hello_world", "helloWorld", Casing::Dromedary);
        test_casing("hello-world", "helloWorld", Casing::Dromedary);
    }

    #[test]
    fn test_pascal_case() {
        test_casing("helloWorld", "HelloWorld", Casing::Pascal);
        test_casing("hello_world", "HelloWorld", Casing::Pascal);
        test_casing("hello-world", "HelloWorld", Casing::Pascal);
    }

    #[test]
    fn test_empty_string() {
        test_casing("", "", Casing::Snake);
        test_casing("", "", Casing::UpperSnake);
        test_casing("", "", Casing::Kebab);
        test_casing("", "", Casing::Dromedary);
        test_casing("", "", Casing::Pascal);
    }

    #[test]
    fn test_single_character() {
        test_casing("a", "a", Casing::Snake);
        test_casing("A", "A", Casing::UpperSnake);
        test_casing("a", "a", Casing::Kebab);
        test_casing("a", "a", Casing::Dromedary);
        test_casing("a", "A", Casing::Pascal);
    }

    #[test]
    fn test_leading_trailing_underscores() {
        test_casing("__hello_world__", "__hello_world__", Casing::Snake);
        test_casing("__hello_world__", "__HELLO_WORLD__", Casing::UpperSnake);
        test_casing("__hello_world__", "__hello-world__", Casing::Kebab);
        test_casing("__hello_world__", "__helloWorld__", Casing::Dromedary);
        test_casing("__hello_world__", "__HelloWorld__", Casing::Pascal);
    }
}
