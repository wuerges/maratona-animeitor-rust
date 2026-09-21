//! Locate resource references without rewriting CSS strings or comments.
//! Network resolution is separate so this tokenizer is tested natively.
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub(super) struct Reference {
    pub span: Range<usize>,
    pub url: String,
    /// Imports retain their layer/supports/media suffix in the original CSS.
    pub import: bool,
}

fn string_end(css: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < css.len() {
        match css[i] {
            b'\\' => i = (i + 2).min(css.len()),
            quote if quote == css[start] => return i + 1,
            _ => i += 1,
        }
    }
    i
}

fn skip_space(css: &[u8], mut i: usize) -> usize {
    loop {
        while i < css.len() && css[i].is_ascii_whitespace() {
            i += 1;
        }
        if css.get(i..i + 2) != Some(b"/*") {
            return i;
        }
        i += 2;
        while i < css.len() && css.get(i..i + 2) != Some(b"*/") {
            i += 1;
        }
        i = (i + 2).min(css.len());
    }
}

fn unescape(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        let Some(c) = chars.next() else {
            break;
        };
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            '\n' | '\x0c' => (),
            c if c.is_ascii_hexdigit() => {
                let mut code = c.to_digit(16).unwrap();
                for _ in 1..6 {
                    match chars.peek().and_then(|c| c.to_digit(16)) {
                        Some(digit) => {
                            code = code * 16 + digit;
                            chars.next();
                        }
                        None => break,
                    }
                }
                if chars.peek().is_some_and(|c| c.is_ascii_whitespace()) {
                    if chars.next() == Some('\r') && chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                }
                result.push(
                    char::from_u32(code)
                        .filter(|c| *c != '\0')
                        .unwrap_or('\u{fffd}'),
                );
            }
            c => result.push(c),
        }
    }
    result
}

fn quoted(css: &str, start: usize) -> Option<(String, usize)> {
    let bytes = css.as_bytes();
    if !matches!(bytes.get(start), Some(b'\'' | b'"')) {
        return None;
    }
    let end = string_end(bytes, start);
    (bytes.get(end.wrapping_sub(1)) == bytes.get(start))
        .then(|| (unescape(&css[start + 1..end - 1]), end))
}

fn url(css: &str, start: usize) -> Option<(String, usize)> {
    let bytes = css.as_bytes();
    if !bytes.get(start..start + 3)?.eq_ignore_ascii_case(b"url") {
        return None;
    }
    let open = skip_space(bytes, start + 3);
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let begin = skip_space(bytes, open + 1);
    let (value, end) = if let Some(quoted) = quoted(css, begin) {
        quoted
    } else {
        let mut end = begin;
        while end < bytes.len() && bytes[end] != b')' {
            end = (end + if bytes[end] == b'\\' { 2 } else { 1 }).min(bytes.len());
        }
        (unescape(css[begin..end].trim()), end)
    };
    let close = skip_space(bytes, end);
    (bytes.get(close) == Some(&b')')).then_some((value, close + 1))
}

pub(super) fn references(css: &str) -> Vec<Reference> {
    let bytes = css.as_bytes();
    let mut refs = vec![];
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i..i + 2) == Some(b"/*") {
            i = skip_space(bytes, i);
            continue;
        }
        if matches!(bytes[i], b'\'' | b'"') {
            i = string_end(bytes, i);
            continue;
        }
        if bytes
            .get(i..i + 7)
            .is_some_and(|s| s.eq_ignore_ascii_case(b"@import"))
        {
            let begin = skip_space(bytes, i + 7);
            if let Some((url, end)) = quoted(css, begin).or_else(|| url(css, begin)) {
                refs.push(Reference {
                    span: begin..end,
                    url,
                    import: true,
                });
                i = end;
                continue;
            }
        }
        let boundary = i == 0
            || !(bytes[i - 1].is_ascii_alphanumeric()
                || matches!(bytes[i - 1], b'_' | b'-' | 128..=255));
        if boundary {
            if let Some((url, end)) = url(css, i) {
                refs.push(Reference {
                    span: i..end,
                    url,
                    import: false,
                });
                i = end;
                continue;
            }
        }
        i += 1;
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_strings_and_comments_and_decodes_escaped_unicode_urls() {
        let css = r#"/* url(missing) */ .a {content: "url(fake)"; background:url('../a\ b.png'); mask:url(#mask); src:url(data:font/woff;base64,AA==); --x:url(\53 ão.png); --y:url(a\)b.png)}"#;
        let refs = references(css);
        assert_eq!(
            refs.iter().map(|r| r.url.as_str()).collect::<Vec<_>>(),
            [
                "../a b.png",
                "#mask",
                "data:font/woff;base64,AA==",
                "São.png",
                "a)b.png"
            ]
        );
    }

    #[test]
    fn imports_preserve_qualifiers_and_accept_comments() {
        let css = "@import/* comment */'nested.css' layer(theme) supports(display: grid) screen;";
        let refs = references(css);
        assert_eq!(refs.len(), 1);
        assert!(refs[0].import);
        assert_eq!(refs[0].url, "nested.css");
        assert_eq!(
            &css[refs[0].span.end..],
            " layer(theme) supports(display: grid) screen;"
        );
    }
}
