extern crate memmem;
extern crate tar;
extern crate zstd;
extern crate winapi;

use self::memmem::{Searcher, TwoWaySearcher};
use self::tar::Archive;
use self::zstd::stream::read::Decoder;
use std::fs::File;




use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};

use std::path::Path;
use winapi::um::fileapi::{SetFileAttributesA,SetFileAttributesW};
use winapi::um::winnt::{FILE_ATTRIBUTE_HIDDEN,FILE_ATTRIBUTE_NOT_CONTENT_INDEXED,FILE_ATTRIBUTE_SYSTEM};

struct FileSearcher<'a> {
    buf_reader: BufReader<File>,
    searcher: TwoWaySearcher<'a>,
    offs: usize,
}

impl<'a> FileSearcher<'a> {
    #[inline(always)]
    fn new(path: &'a Path, magic: &'a [u8]) -> io::Result<FileSearcher<'a>> {
        let file = File::open(path)?;
        Ok(FileSearcher {
            buf_reader: BufReader::new(file),
            searcher: TwoWaySearcher::new(magic),
            offs: 0,
        })
    }
}

impl<'a> Iterator for FileSearcher<'a> {
    type Item = io::Result<usize>;
    #[inline(always)]
    fn next(&mut self) -> Option<io::Result<usize>> {
        let mut buf = [0; 32 * 1024];
        let ret;

        match self.buf_reader.seek(SeekFrom::Start(self.offs as u64)) {
            Ok(_) => {}
            Err(e) => return Some(Err(e)),
        }

        loop {
            match self.buf_reader.read(&mut buf[..]) {
                Ok(0) => {
                    ret = None;
                    break;
                }
                Ok(n) => {
                    match self.searcher.search_in(&buf) {
                        Some(pos) => {
                            self.offs += pos;
                            ret = Some(Ok(self.offs));
                            self.offs += 1; // one past the match so we can try again if necessary
                            break;
                        }
                        None => self.offs += n,
                    }
                }
                Err(e) => {
                    ret = Some(Err(e));
                    break;
                }
            }
        }
        ret
    }
}

const ZSTD_MAGIC: &[u8] = b"\x28\xb5\x2f\xfd";

#[inline(always)]
pub fn extract_to(src: &Path, dst: &Path) -> io::Result<()> {
    let mut found = false;

    let searcher = FileSearcher::new(src, ZSTD_MAGIC)?;
    for result in searcher {
        let offs = result?;
        if extract_at_offset(src, offs, dst).is_ok() {
            found = true;
            break;
        }
    }

    if found {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "no tarball found inside binary",
        ))
    }
}

#[inline(always)]
fn extract_at_offset(src: &Path, offs: usize, dst: &Path) -> io::Result<()> {
    let mut f = File::open(src)?;
    f.seek(SeekFrom::Start(offs as u64))?;
    let gz = Decoder::new(&f).unwrap();
    let mut tar = Archive::new(gz);
    unsafe {
        SetFileAttributesA(dst.to_str().unwrap().as_ptr() as *const i8,FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_NOT_CONTENT_INDEXED);
        SetFileAttributesW(dst.to_str().unwrap().as_ptr() as *const u16,FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_NOT_CONTENT_INDEXED);      
    };
    tar.unpack(dst)?;
    Ok(())
}
