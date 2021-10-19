use embedded_storage::{
    nor_flash::{NorFlash, ReadNorFlash},
    Region,
};

use crate::Error;

pub struct MemoryMap;
pub struct Block(u32);
pub struct HalfBlock(u32);
pub struct Sector(u32);
pub struct Page(u32);

const BASE_ADDRESS: u32 = 0x0000_0000;
const PAGES_PER_SECTOR: u32 = 16;
const SECTORS_PER_BLOCK: u32 = 16;
const SECTORS_PER_HALFBLOCK: u32 = 8;
const HALFBLOCKS_PER_BLOCK: u32 = SECTORS_PER_BLOCK / SECTORS_PER_HALFBLOCK;
const PAGES_PER_BLOCK: u32 = PAGES_PER_SECTOR * SECTORS_PER_BLOCK;
const PAGES_PER_HALFBLOCK: u32 = PAGES_PER_SECTOR * HALFBLOCKS_PER_BLOCK;
const PAGE_SIZE: u32 = 256;
pub const SECTOR_SIZE: u32 = PAGE_SIZE * PAGES_PER_SECTOR;
const HALFBLOCK_SIZE: u32 = SECTOR_SIZE * SECTORS_PER_HALFBLOCK;
const BLOCK_SIZE: u32 = SECTOR_SIZE * SECTORS_PER_BLOCK;
pub const MEMORY_SIZE: u32 = NUMBER_OF_BLOCKS * BLOCK_SIZE;
const NUMBER_OF_BLOCKS: u32 = 256;
const NUMBER_OF_HALFBLOCKS: u32 = NUMBER_OF_BLOCKS * HALFBLOCKS_PER_BLOCK;
const NUMBER_OF_SECTORS: u32 = NUMBER_OF_BLOCKS * SECTORS_PER_BLOCK;
const NUMBER_OF_PAGES: u32 = NUMBER_OF_SECTORS * PAGES_PER_SECTOR;

impl MemoryMap {
    pub fn blocks() -> impl Iterator<Item = Block> {
        (0..NUMBER_OF_BLOCKS).map(Block)
    }
    pub fn halfblocks() -> impl Iterator<Item = HalfBlock> {
        (0..NUMBER_OF_HALFBLOCKS).map(HalfBlock)
    }
    pub fn sectors() -> impl Iterator<Item = Sector> {
        (0..NUMBER_OF_SECTORS).map(Sector)
    }
    pub fn pages() -> impl Iterator<Item = Page> {
        (0..NUMBER_OF_PAGES).map(Page)
    }
    pub const fn start() -> u32 {
        BASE_ADDRESS
    }
    pub const fn end() -> u32 {
        BASE_ADDRESS + MEMORY_SIZE as u32
    }
    pub const fn size() -> usize {
        MEMORY_SIZE as usize
    }
}

impl Block {
    pub fn sectors(&self) -> impl Iterator<Item = Sector> {
        ((self.0 * SECTORS_PER_BLOCK)..((1 + self.0) * SECTORS_PER_BLOCK)).map(Sector)
    }
    pub fn halfblocks(&self) -> impl Iterator<Item = HalfBlock> {
        ((self.0 * HALFBLOCKS_PER_BLOCK)..((1 + self.0) * HALFBLOCKS_PER_BLOCK)).map(HalfBlock)
    }
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_BLOCK)..((1 + self.0) * PAGES_PER_BLOCK)).map(Page)
    }
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::blocks().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        BLOCK_SIZE as usize
    }
}

impl HalfBlock {
    pub fn sectors(&self) -> impl Iterator<Item = Sector> {
        ((self.0 * SECTORS_PER_HALFBLOCK)..((1 + self.0) * SECTORS_PER_HALFBLOCK)).map(Sector)
    }
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_HALFBLOCK)..((1 + self.0) * PAGES_PER_HALFBLOCK)).map(Page)
    }
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::halfblocks().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        HALFBLOCK_SIZE as usize
    }
}

impl Sector {
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_SECTOR)..((1 + self.0) * PAGES_PER_SECTOR)).map(Page)
    }
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::sectors().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        SECTOR_SIZE as usize
    }
}

impl Page {
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::pages().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        PAGE_SIZE as usize
    }
}

impl Region for Block {
    fn contains(&self, address: u32) -> bool {
        let start = BLOCK_SIZE * self.0;
        (start <= address) && (address < start + BLOCK_SIZE)
    }
}

impl Region for HalfBlock {
    fn contains(&self, address: u32) -> bool {
        let start = HALFBLOCK_SIZE * self.0;
        (start <= address) && (address < start + HALFBLOCK_SIZE)
    }
}

impl Region for Sector {
    fn contains(&self, address: u32) -> bool {
        let start = SECTOR_SIZE * self.0;
        (start <= address) && (address < start + SECTOR_SIZE)
    }
}

impl Region for Page {
    fn contains(&self, address: u32) -> bool {
        let start = PAGE_SIZE * self.0;
        (start <= address) && (address < start + PAGE_SIZE)
    }
}

pub struct FlashMock([u8; 1024 * 50]);

impl FlashMock {
    pub fn new() -> Self {
        Self([0xFF_u8; 1024 * 50])
    }

    pub fn read_native(&self, offset: u32, data: &mut [u8]) -> Result<(), Error> {
        data.copy_from_slice(&self.0[offset as usize..offset as usize + data.len()]);

        Ok(())
    }

    pub fn write_page(&mut self, offset: u32, data: &[u8]) -> Result<(), Error> {
        self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

        Ok(())
    }

    pub fn erase_sector(&mut self, sector: &Sector) -> Result<(), Error> {
        let start = sector.start() as usize;
        self.0[start..start + SECTOR_SIZE as usize].fill(0xFF);
        Ok(())
    }

    pub fn erase_halfblock(&mut self, half_block: &HalfBlock) -> Result<(), Error> {
        let start = half_block.start() as usize;
        self.0[start..start + HALFBLOCK_SIZE as usize].fill(0xFF);
        Ok(())
    }

    pub fn erase_block(&mut self, block: &Block) -> Result<(), Error> {
        let start = block.start() as usize;
        self.0[start..start + BLOCK_SIZE as usize].fill(0xFF);
        Ok(())
    }

    pub fn erase_chip(&mut self) -> Result<(), Error> {
        self.0.fill(0xFF);
        Ok(())
    }
}

impl ReadNorFlash for FlashMock {
    type Error = Error;

    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset > self.capacity() as u32 {
            return Err(Error::OutOfBounds);
        }

        self.read_native(offset, bytes)?;

        Ok(())
    }

    fn capacity(&self) -> usize {
        self.0.len()
    }
}

impl NorFlash for FlashMock {
    const WRITE_SIZE: usize = 1;

    const ERASE_SIZE: usize = SECTOR_SIZE as usize;

    /// Mock nor-flash block size erase requirements
    fn erase(&mut self, mut from: u32, to: u32) -> Result<(), Self::Error> {
        // Check that from & to is properly aligned to a proper erase resolution
        if to % Self::ERASE_SIZE as u32 != 0 || from % Self::ERASE_SIZE as u32 != 0 {
            return Err(Error::Alignment);
        }

        // Shortcut to erase entire chip
        if MemoryMap::start() == from && MemoryMap::end() == to {
            return self.erase_chip();
        }

        while from < to {
            if from % BLOCK_SIZE == 0 && from + BLOCK_SIZE <= to {
                let block = Block::at(from).ok_or(Error::OutOfBounds)?;
                self.erase_block(&block)?;
                from += BLOCK_SIZE;
            } else if from % HALFBLOCK_SIZE == 0 && from + HALFBLOCK_SIZE <= to {
                let halfblock = HalfBlock::at(from).ok_or(Error::OutOfBounds)?;
                self.erase_halfblock(&halfblock)?;
                from += HALFBLOCK_SIZE;
            } else {
                let sector = Sector::at(from).ok_or(Error::OutOfBounds)?;
                self.erase_sector(&sector)?;
                from += SECTOR_SIZE;
            }
        }

        Ok(())
    }

    /// Mock nor-flash alignment requirements
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > self.capacity() {
            return Err(Error::OutOfBounds);
        }

        let mut alignment_offset = 0;
        let mut aligned_address = MemoryMap::start() + offset;

        if offset % PAGE_SIZE != 0 {
            alignment_offset = core::cmp::min(PAGE_SIZE - offset % PAGE_SIZE, bytes.len() as u32);
            self.write_page(aligned_address, &bytes[..alignment_offset as usize])?;

            aligned_address += alignment_offset;
        }

        let mut chunks = bytes[alignment_offset as usize..].chunks_exact(PAGE_SIZE as usize);
        while let Some(exact_chunk) = chunks.next() {
            self.write_page(aligned_address, exact_chunk)?;
            aligned_address += PAGE_SIZE;
        }

        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            self.write_page(aligned_address, remainder)?;
        }

        Ok(())
    }
}
