use {
    memmap::{MmapOptions, Mmap},
    std::{
        str::Utf8Error,
        fs::File,
    }
};

#[derive(Clone, Copy, Debug)]
pub struct CharPosition {
    pub byte_position: usize,
    pub char_position: usize,
}

/// A Memory Mapped File
pub struct MappedFile {
    pub file: File,
    pub map: Mmap,
    pub line_ending_indexes: Vec<CharPosition>,
}

#[derive(Debug)]
pub enum IndexError {
    OutOfBounds,
    InvalidChar(Utf8Error),
}

impl MappedFile {
    pub fn new(file: File) -> Result<MappedFile, String> {
        let map = unsafe { MmapOptions::new().map(&file).map_err(|e| e.to_string())? };
        Ok(MappedFile { file, map, line_ending_indexes: vec![CharPosition {char_position: 0, byte_position: 0}] })
    }

    /// Returns the index of the line ending at the given byte position.
    /// Returns an error if the byte position is out of bounds.

    pub fn unicode_at(&mut self, index: usize) -> Result<char, IndexError> {
        let index= index + 1;

        // Check through to see if we have something close to the index in the line cache
        for window in self.line_ending_indexes.windows(2) {
            let (last, current) = (window[0], window[1]);

            // if we do, we can locate it in a line
            if last.char_position < index && index <= current.char_position {
                return self.get_unicode_char(last.byte_position, index - last.char_position, current.byte_position);
            }
        }

        // We don't have the closest line, so we need to go through the file until we find the index
        if let Some(current) = self.line_ending_indexes.last() {
            // Parse the file to utf8
            let temp_str = std::str::from_utf8(&self.map[current.byte_position..]).unwrap();

            // Set the initial char and byte position
            let mut char_index = current.char_position;
            let mut byte_index = current.byte_position;

            // Go through the file until we find the index
            for c in temp_str.chars() {
                // update the positions
                char_index += 1;
                byte_index += c.len_utf8();


                // if we have a newline we need to update the line ending indexes
                if c == '\n' {
                    self.line_ending_indexes.push(CharPosition { byte_position: byte_index, char_position: char_index });
                }

                // if we have found the index, return the char
                if char_index == index  {
                    return Ok(c);
                }
            }
        }

        // We haven't found the character at the index so it isn't in the file
        Err(IndexError::OutOfBounds)
    }

    pub fn get_unicode_char(&self, byte_position: usize, index: usize, next_line_byte_position: usize) -> Result<char, IndexError> {
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
mod tests {
}
