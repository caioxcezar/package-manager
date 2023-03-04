pub fn split_utf8(text: &str, start: usize, end: usize) -> String {
    text.chars().take(end).skip(start).collect::<String>().trim().to_owned()
}