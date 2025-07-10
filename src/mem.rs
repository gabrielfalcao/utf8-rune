use core::alloc::Layout;
use core::fmt::{Debug, Display, Formatter};

use crate::Result;

#[inline]
pub fn layout(requested_size: usize) -> Result<Layout> {
    let actual_size = if requested_size == 0 {
        1
    } else {
        requested_size
    };
    Ok(Layout::array::<u8>(actual_size)
        .map_err(|e| MemoryError::from_layout_error(e, requested_size, actual_size))?)
}

#[derive(Clone, PartialEq, Eq)]
pub struct MemoryError {
    pub requested_size: usize,
    pub actual_size: usize,
    pub message: String,
}

impl std::error::Error for MemoryError {}

impl MemoryError {
    pub fn new<T: Display>(
        message: T,
        requested_size: usize,
        actual_size: usize,
    ) -> MemoryError {
        let message = message.to_string();
        MemoryError {
            message,
            requested_size,
            actual_size,
        }
    }

    pub fn from_layout_error(
        e: std::alloc::LayoutError,
        requested_size: usize,
        actual_size: usize,
    ) -> MemoryError {
        let s = if actual_size != 1 {
            "s"
        } else {
            ""
        }
        .to_string();
        let contiguous = if actual_size != 1 {
            "contiguous"
        } else {
            ""
        }
        .to_string();
        let error = format!(": {e}");
        let bytes = format!("u8 byte{s}");
        MemoryError {
            requested_size,
            actual_size,
            message: [
                format!("failed to obtain a memory layout of {actual_size}"),
                contiguous,
                bytes,
                error,
            ]
            .into_iter()
            .filter(|c| !c.is_empty())
            .collect::<Vec<String>>()
            .join(" "),
        }
    }
}

impl Debug for MemoryError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let message = &self.message;
        let requested_size = self.requested_size;
        let actual_size = self.actual_size;
        write!(f, "MemoryError{{requested_size: {requested_size}, actual_size: {actual_size}}}: {message}")
    }
}
impl Display for MemoryError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let message = &self.message;
        write!(f, "MemoryError: {message}")
    }
}
