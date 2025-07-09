#[derive(Clone, PartialEq, Eq)]
pub enum Error<'g> {
    InvalidIndex(usize, &'g [u8]),
    UnexpectedContinuationByte(u8, usize, Option<usize>, Option<usize>, &'g [u8]),
    Utf8Error(usize, &'g [u8], String),
}
impl<'g> Error<'g> {
    pub fn previous_valid_cutoff(&self) -> Option<usize> {
        match self {
            Error::InvalidIndex(_, _) => None,
            Error::UnexpectedContinuationByte(_, _, previous, _, _) => previous.clone(),
            Error::Utf8Error(_, _, _) => None,
        }
    }

    pub fn next_valid_cutoff(&self) -> Option<usize> {
        match self {
            Error::InvalidIndex(_, _) => None,
            Error::UnexpectedContinuationByte(_, _, _, next, _) => next.clone(),
            Error::Utf8Error(_, _, _) => None,
        }
    }
}
impl<'g> std::fmt::Display for Error<'g> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn format_slice(slice: &[u8], index: usize) -> String {
            format!(
                "[{}]",
                slice
                    .into_iter()
                    .enumerate()
                    .map(|(i,byte)| if i == index {
                        format!("\x1b[1;38;5;220m0x{byte:02x}\x1b[0m")
                    } else {
                        format!("0x{byte:02x}")

                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
        write!(
            f,
            "{}",
            match self {
                Error::InvalidIndex(index, slice) => {
                    let length = slice.len();
                    format!(
                        "invalid index {index}: {index} > {length} in {}",
                        format_slice(slice, *index)
                    )
                },
                Error::Utf8Error(index, slice, error) => {
                    format!(
                        "Utf8Error in index {index} of {}: {error}",
                        format_slice(slice, *index)
                    )
                },
                Error::UnexpectedContinuationByte(
                    byte,
                    index,
                    previous_valid_cutoff,
                    next_valid_cutoff,
                    slice,
                ) => {
                    [
                        Some(format!(
                        "unexpected continuation byte 0x{byte:02x}(0b{byte:08b}) at index {index} of {}",format_slice(slice, *index)
                        )),
                        previous_valid_cutoff.map(|previous|format!("previous valid cutoff index: {previous}")),
                        next_valid_cutoff.map(|next|format!("next valid cutoff index: {next}")),
                    ].into_iter().filter(|c|c.is_some()).map(|c|c.unwrap().to_string()).collect::<Vec<String>>().join("\n")
                },
            }
        )
    }
}
impl<'g> std::fmt::Debug for Error<'g> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
impl<'g> std::error::Error for Error<'g> {}
pub type Result<T> = std::result::Result<T, Error<'static>>;
