use regex::Regex;

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

pub fn pad_volume_number(input: &str) -> String {
    let re = Regex::new(r"\s*\(?\s*(\d+)\s*\)?\s+").unwrap();
    let result = re.replace_all(input, |caps: &regex::Captures| {
        format!(" {:02} ", caps[1].parse::<i32>().unwrap())
    });

    let re = Regex::new(r"\s*\(?\s*(\d+)\s*\)?\s*$").unwrap();
    let result = re.replace_all(&result, |cap: &regex::Captures| {
        format!(" {:02}", cap[1].parse::<i32>().unwrap())
    });

    let re = Regex::new(r"第?\s*(\d+)巻?\s*$").unwrap();
    let result = re.replace_all(&result, |cap: &regex::Captures| {
        format!(" {:02}", cap[1].parse::<i32>().unwrap())
    });

    result.to_string()
}

pub fn remove_spaces(input: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    let result = re.replace_all(input, " ");

    result.trim().to_string()
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

    #[test]
    fn test_pad_volume_number() {
        assert_eq!(super::pad_volume_number("xxx (1) yyy"), "xxx 01 yyy");
        assert_eq!(super::pad_volume_number("xxx(1) yyy"), "xxx 01 yyy");
        assert_eq!(super::pad_volume_number("xxx (1)"), "xxx 01");
        assert_eq!(super::pad_volume_number("xxx 1 yyy"), "xxx 01 yyy");
        assert_eq!(super::pad_volume_number("xxx1 yyy"), "xxx 01 yyy");
        assert_eq!(super::pad_volume_number("xxx 1"), "xxx 01");
        assert_eq!(super::pad_volume_number("xxx1巻"), "xxx 01");
        assert_eq!(super::pad_volume_number("xxx第1巻"), "xxx 01");
        assert_eq!(super::pad_volume_number("xxx1"), "xxx 01");
    }

    #[test]
    fn test_remove_spaces() {
        assert_eq!(super::remove_spaces("  xxx  yyy  "), "xxx yyy");
    }
}
