use anyhow::{Context, Result};
use secstr::SecVec;

pub fn split_utf8(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .take(end)
        .skip(start)
        .collect::<String>()
        .trim()
        .to_owned()
}
pub fn pass_2_stdin(password: Option<SecVec<u8>>) -> Result<Vec<String>> {
    let password = String::from_utf8(password.context("Missing password")?.unsecure().to_vec())?;
    Ok([password].to_vec())
}
