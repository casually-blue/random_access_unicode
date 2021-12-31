use {
    memmap::{Mmap, MmapOptions},
    std::{fs::File, str::Utf8Error},
};

#[derive(Clone, Copy, Debug)]
/// The position of a character in a file as both a character index and a byte index of the start of the character.
pub struct CharPosition {
    // The position of the character in bytes
    pub byte_position: usize,
    // The position of the character in utf8 characters
    pub char_position: usize,
}

/// A Memory Mapped File
pub struct MappedFile {
    /// The file that the memory map is mapped to
    pub file: File,
    /// The memory map of the file
    pub map: Mmap,

    /// The cache of line ending positions
    pub line_ending_positions: Vec<CharPosition>,
}

#[derive(Debug)]
pub enum IndexError {
    /// The index is outside of the bounds of the file
    OutOfBounds,
    /// The index is not a valid utf8 character
    InvalidChar(Utf8Error),
}

impl MappedFile {
    /// Creates a new MappedFile from a File
    /// possibly returning an error
    pub fn new(file: File) -> Result<MappedFile, String> {
        let map = unsafe { MmapOptions::new().map(&file).map_err(|e| e.to_string())? };
        Ok(MappedFile {
            file,
            map,
            // First line ending is the start of the file
            // Mainly so I can just get the last element regardless of whether i have encountered any line endings yet
            line_ending_positions: vec![CharPosition {
                char_position: 0,
                byte_position: 0,
            }],
        })
    }

    fn find_with_cache(&mut self, index: usize) -> Result<char, IndexError> {
        for window in self.line_ending_positions.windows(2) {
            let (last, current) = (window[0], window[1]);

            // Check if the index is in the current window
            if last.char_position < index && index <= current.char_position {
                // If it is, we locate it in the map
                return self.find_nth_in_str(
                    index - last.char_position,
                    last,
                    Some(current),
                );
            }
        }
        
        // If we get here, it means that the index is outside of the bounds of the line
        Err(IndexError::OutOfBounds)
    }

    fn find_nth_in_str(
        &mut self,
        n: usize,
        start: CharPosition,
        end: Option<CharPosition>,
    ) -> Result<char, IndexError> {
        let str = match std::str::from_utf8(match end {
            Some(end) => &self.map[start.byte_position..end.byte_position],
            None => &self.map[start.byte_position..],
        }) {
            Ok(s) => s,
            Err(e) => return Err(IndexError::InvalidChar(e)),
        };

        // If we know we're inside a line, we can just get the nth character
        if let Some(_) = end {
            Ok(str.chars().nth(n).unwrap())
        // Otherwise we have to iterate and update the cache
        } else {
            let mut byte_position = start.byte_position;
            for (char_index, c) in str.chars().enumerate() {
                // update the positions
                byte_position += c.len_utf8();

                // if we have a newline we need to update the line ending indexes
                if c == '\n' {
                    self.line_ending_positions.push(CharPosition {
                        byte_position: byte_position,
                        char_position: char_index + start.char_position,
                    });
                }

                // if we have found the index, return the char
                if char_index == n {
                    return Ok(c);
                }
            }

            // if we get here, we didn't find the index
            Err(IndexError::OutOfBounds)
        }
    }

    /// Returns the index of the line ending at the given byte position.
    /// Returns an error if the byte position is out of bounds.
    pub fn unicode_at(&mut self, index: usize) -> Result<char, IndexError> {
        let index = index + 1;

        // Check through to see if we have something close to the index in the line cache
        if let Ok(c) = self.find_with_cache(index) {
            return Ok(c);
        } else {
            let current = self.line_ending_positions.last().cloned().unwrap();
            // Go through the file until we find the index
            self.find_nth_in_str(index - current.char_position, current, None)
        }
    }
}

#[cfg(test)]
mod tests {}
