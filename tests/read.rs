use std::fs::File;
use std::io::Write;

use random_access_unicode::*;

#[test]
pub fn test_helloworld() {
    let mut file = File::create("test.txt").unwrap();
    write!(file, "Hello\nworld!\n").unwrap();
    file.flush().unwrap();

    let mut r = MappedFile::new(File::open("test.txt").unwrap()).unwrap();

    assert_eq!(r.unicode_at(0).unwrap(), 'H');
    assert_eq!(r.unicode_at(1).unwrap(), 'e');
    assert_eq!(r.unicode_at(2).unwrap(), 'l');
    assert_eq!(r.unicode_at(3).unwrap(), 'l');
    assert_eq!(r.unicode_at(4).unwrap(), 'o');
    assert_eq!(r.unicode_at(5).unwrap(), '\n');
    assert_eq!(r.unicode_at(6).unwrap(), 'w');
    assert_eq!(r.unicode_at(7).unwrap(), 'o');
    assert_eq!(r.unicode_at(8).unwrap(), 'r');
    assert_eq!(r.unicode_at(9).unwrap(), 'l');
    assert_eq!(r.unicode_at(10).unwrap(), 'd');
    assert_eq!(r.unicode_at(11).unwrap(), '!');
    assert_eq!(r.unicode_at(12).unwrap(), '\n');
    
}