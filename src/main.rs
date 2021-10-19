#![feature(lang_items, start, libc, core_intrinsics, rustc_private)]
#![no_std]

mod flash;

extern crate libc;

use core::panic::PanicInfo;

use embedded_storage::{nor_flash::RmwNorFlashStorage, Storage};
use fatfs::{FileSystem, FsOptions, IoBase, IoError, Read, Seek, Write};
use flash::{FlashMock, SECTOR_SIZE};
use libc_print::{libc_print, libc_println};
use log::{debug, error, info, trace, LevelFilter};
use log::{Log, Metadata, Record};

use crate::flash::MEMORY_SIZE;

#[derive(Debug)]
pub enum FsError<S: core::fmt::Debug> {
    Storage(S),
    OutOfBounds,
    WriteZero,
    Eof,
}

impl<S: core::fmt::Debug> IoError for FsError<S> {
    fn is_interrupted(&self) -> bool {
        error!("WTF is_interrupted");
        false
    }

    fn new_unexpected_eof_error() -> Self {
        error!("WTF new_unexpected_eof_error");
        Self::Eof
    }

    fn new_write_zero_error() -> Self {
        error!("WTF new_write_zero_error");
        Self::WriteZero
    }
}

pub struct FatIO<T> {
    inner: T,
    pos: u32,
}

impl<T> FatIO<T> {
    pub fn new(storage: T) -> Self {
        Self {
            inner: storage,
            pos: 0,
        }
    }
}

impl<T> IoBase for FatIO<T>
where
    T: Storage,
    T::Error: core::fmt::Debug,
{
    type Error = FsError<T::Error>;
}

impl<T> Seek for FatIO<T>
where
    T: Storage,
    T::Error: core::fmt::Debug,
{
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
        let (base_pos, offset) = match pos {
            fatfs::SeekFrom::Start(n) => {
                self.pos = n as u32;
                return Ok(self.pos as u64);
            }
            fatfs::SeekFrom::End(n) => (self.inner.capacity() as u32, n),
            fatfs::SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as u32)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as u32)
        };
        match new_pos {
            Some(n) => {
                self.pos = n as u32;
                Ok(self.pos as u64)
            }
            None => Err(FsError::OutOfBounds),
        }
    }
}

impl<T> Read for FatIO<T>
where
    T: Storage,
    T::Error: core::fmt::Debug,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.read(self.pos, buf).map_err(FsError::Storage)?;

        self.pos += buf.len() as u32;

        Ok(buf.len())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.read(buf)?;
        Ok(())
    }
}

impl<T> Write for FatIO<T>
where
    T: Storage,
    T::Error: core::fmt::Debug,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.inner.write(self.pos, buf).map_err(FsError::Storage)?;

        self.pos += buf.len() as u32;
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.write(buf)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    OutOfBounds,
    Alignment,
}

// A list of C functions that are being imported
extern "C" {
    pub fn printf(format: *const u8, ...) -> i32;
}

pub struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            libc_println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOG: SimpleLogger = SimpleLogger;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);

    loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    log::set_logger(&LOG).ok();
    log::set_max_level(LevelFilter::Trace);

    let mut merge_buffer = [0u8; SECTOR_SIZE as usize];

    let flash = FlashMock::new();

    let mut fat_io = FatIO::new(RmwNorFlashStorage::new(flash, &mut merge_buffer));

    fatfs::format_volume(&mut fat_io, fatfs::FormatVolumeOptions::new()).expect("format volume");

    let mut fs = FileSystem::new(fat_io, FsOptions::new()).unwrap();

    if let Ok(mut file) = fs.root_dir().create_file("index.html.gz") {
        file.write_all(b"this is the file contents of my index file").expect("Faield to write");
    }

    fs.root_dir().open_file("index.html.gz").unwrap();

    loop {}
}
