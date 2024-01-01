pub fn to_half_width(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if (0xFF01..=0xFF5E).contains(&(c as u32)) {
                (c as u32 - 0xFF00 + 0x20) as u8 as char
            } else {
                c
            }
        })
        .collect()
}

pub fn replace_unsafe_symbols(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '<' => '＜',
            '>' => '＞',
            ':' => '：',
            '"' => '＂',
            '/' => '／',
            '\\' => '￥',
            '!' => '！',
            '?' => '？',
            '*' => '＊',
            _ => c,
        })
        .collect()
}

pub fn replace_round_brackets(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '（' => '(',
            '）' => ')',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_to_half_width() {
        let actual = super::to_half_width("ａｚ1ＡＺa０９");
        let expected = "az1AZa09";
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_replace_unsafe_symbols() {
        let actual = super::replace_unsafe_symbols(r#"<>:"/\!?*d"#);
        let expected = "＜＞：＂／￥！？＊d";
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_replace_round_brackets() {
        let actual = super::replace_round_brackets("（）");
        let expected = "()";
        assert_eq!(actual, expected)
    }
}
