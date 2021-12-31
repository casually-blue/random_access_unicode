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
            line_ending_positions: vec![CharPosition {
                char_position: 0,
                byte_position: 0,
            }],
        })
    }

    fn find_with_cache(&self, index: usize) -> Option<char> {
        for window in self.line_ending_positions.windows(2) {
            let (last, current) = (window[0], window[1]);

            // if we do, we can locate it in a line
            if last.char_position < index && index <= current.char_position {
                return match self.get_unicode_char_in_line(
                    last.byte_position,
                    index - last.char_position,
                    current.byte_position,
                ) {
                    Ok(c) => Some(c),
                    Err(_) => None,
                };
            }
        }
        None
    }

    fn find_nth_in_str(&mut self, n: usize, start: CharPosition) -> Option<char> {
        let str = std::str::from_utf8(&self.map[start.byte_position..]).unwrap();

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
                return Some(c);
            }
        }

        // if we get here, we didn't find the index
        None
    }

    /// Returns the index of the line ending at the given byte position.
    /// Returns an error if the byte position is out of bounds.
    pub fn unicode_at(&mut self, index: usize) -> Result<char, IndexError> {
        let index = index + 1;

        // Check through to see if we have something close to the index in the line cache
        if let Some(c) = self.find_with_cache(index) {
            return Ok(c);
        } else {
            match self.line_ending_positions.last().cloned() {
                Some(current) => {
                    // Go through the file until we find the index
                    match self.find_nth_in_str(index - current.char_position, current) {
                        Some(c) => Ok(c),
                        None => Err(IndexError::OutOfBounds),
                    }
                },
                None => Err(IndexError::OutOfBounds),
            }
        }
    }

    /// Gets the char at the given index in the given line
    fn get_unicode_char_in_line(
        &self,
        byte_position: usize,
        index: usize,
        next_line_byte_position: usize,
    ) -> Result<char, IndexError> {
        // Get the current line as a slice
        let slice = &self.map[byte_position..next_line_byte_position];

        // Parse the slice as utf8
        let temp_str = match std::str::from_utf8(slice) {
            Ok(s) => s,
            // If the slice isn't valid utf8, return an error
            Err(err) => return Err(IndexError::InvalidChar(err)),
        };

        // Get the char at the index
        match temp_str.chars().nth(index) {
            Some(c) => Ok(c),
            // If the index is out of bounds, return an error
            None => Err(IndexError::OutOfBounds),
        }
    }
}

#[cfg(test)]
mod tests {}
