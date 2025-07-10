use std::iter::Iterator;

pub const DEFAULT_INDENT: usize = 4;
use crate::Error;

#[cfg(feature = "debug")]
pub fn format_bytes(bytes: &[u8], indent: Option<usize>) -> String {
    let indent = indent.unwrap_or_else(|| DEFAULT_INDENT);
    let padding = " ".repeat(indent);
    fn pad(byte: u8, indent: usize) -> String {
        let byte = byte.to_string();
        let pad = " ".repeat(indent - (1 - byte.len()));
        format!("{byte}{pad}")
    }
    format!(
        "[\n{}{}\n]",
        padding,
        bytes
            .iter()
            .map(Clone::clone)
            .map(|c| format!(
                "{}{}, // {:#?}",
                " ".repeat(indent + DEFAULT_INDENT),
                pad(c, indent),
                char::from(c)
            ))
            .collect::<Vec<String>>()
            .join("\n"),
    )
}
#[cfg(not(feature = "debug"))]
pub fn format_bytes(bytes: &[u8], indent: Option<usize>) -> String {
    let pad = " ".repeat(unwrap_indent(indent));
    format!(
        "{pad}[{}]",
        bytes
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    )
}

#[cfg(feature = "debug")]
pub fn display_error<'e>(error: Error<'e>, ptr: *const u8, length: usize) {
    let filename = file!();
    let lineno = line!();
    eprintln!("{filename}:{lineno} {error}");
}
#[cfg(not(feature = "debug"))]
pub fn display_error<'e>(_error: Error<'e>, _ptr: *const u8, _length: usize) {}


pub fn unwrap_indent(indent: Option<usize>) -> usize {
    indent.unwrap_or_else(|| DEFAULT_INDENT)
}
