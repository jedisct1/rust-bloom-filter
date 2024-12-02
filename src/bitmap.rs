use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;

pub const VERSION: u8 = 1;
pub const BITMAP_HEADER_SIZE: usize = 1 + 8 + 4 + 32;

#[derive(Clone, Debug)]
pub(crate) struct BitMap {
    header_and_bits: Vec<u8>,
}

impl BitMap {
    pub fn new(len_bytes: usize) -> Self {
        let mut header_and_bits = vec![0; BITMAP_HEADER_SIZE + len_bytes];
        let header = &mut header_and_bits[0..BITMAP_HEADER_SIZE];
        Self::set_version(header, VERSION);
        Self::set_len_bytes(header, len_bytes as u64);
        Self::set_k_num(header, 0);
        Self { header_and_bits }
    }

    fn bits(&self) -> &[u8] {
        &self.header_and_bits[BITMAP_HEADER_SIZE..]
    }

    fn bits_mut(&mut self) -> &mut [u8] {
        &mut self.header_and_bits[BITMAP_HEADER_SIZE..]
    }

    pub fn header(&self) -> &[u8] {
        &self.header_and_bits[0..BITMAP_HEADER_SIZE]
    }

    pub fn header_mut(&mut self) -> &mut [u8] {
        &mut self.header_and_bits[0..BITMAP_HEADER_SIZE]
    }

    fn get_version(header: &[u8]) -> u8 {
        header[0]
    }

    fn set_version(header: &mut [u8], version: u8) {
        header[0] = version;
    }

    fn get_len_bytes(header: &[u8]) -> u64 {
        u64::from_le_bytes(header[1..][0..8].try_into().unwrap())
    }

    fn set_len_bytes(header: &mut [u8], len_bytes: u64) {
        header[1..][0..8].copy_from_slice(&len_bytes.to_le_bytes());
    }

    pub fn get_k_num(header: &[u8]) -> u32 {
        u32::from_le_bytes(header[9..][0..4].try_into().unwrap())
    }

    pub fn set_k_num(header: &mut [u8], k_num: u32) {
        header[9..][0..4].copy_from_slice(&k_num.to_le_bytes());
    }

    pub fn get_seed(header: &[u8]) -> [u8; 32] {
        header[13..][0..32].try_into().unwrap()
    }

    pub fn set_seed(header: &mut [u8], seed: &[u8; 32]) {
        header[13..][0..32].copy_from_slice(seed);
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if bytes.len() < BITMAP_HEADER_SIZE {
            return Err("Invalid size");
        }
        let header = &bytes[0..BITMAP_HEADER_SIZE];
        let bits = &bytes[BITMAP_HEADER_SIZE..];
        if Self::get_version(header) != VERSION {
            return Err("Version mismatch");
        }
        if Self::get_k_num(header) == 0 {
            return Err("Invalid number of keys");
        }
        let len_bytes_u64 = Self::get_len_bytes(header);
        let len_bytes: usize = len_bytes_u64.try_into().map_err(|_| "Too big")?;
        if bits.len() != len_bytes {
            return Err("Invalid size");
        }
        let res = Self {
            header_and_bits: bytes,
        };
        Ok(res)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < BITMAP_HEADER_SIZE {
            return Err("Invalid size");
        }
        let header = &bytes[0..BITMAP_HEADER_SIZE];
        let bits = &bytes[BITMAP_HEADER_SIZE..];
        if Self::get_version(header) != VERSION {
            return Err("Version mismatch");
        }
        if Self::get_k_num(header) == 0 {
            return Err("Invalid number of keys");
        }
        let len_bytes_u64 = Self::get_len_bytes(header);
        let len_bytes: usize = len_bytes_u64.try_into().map_err(|_| "Too big")?;
        if bits.len() != len_bytes {
            return Err("Invalid size");
        }
        let res = Self {
            header_and_bits: bytes.to_vec(),
        };
        Ok(res)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.header_and_bits
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.header_and_bits
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.header_and_bits.clone()
    }

    pub fn get(&self, bit_offset: usize) -> bool {
        let byte_offset = bit_offset / 8;
        let bit_shift = bit_offset % 8;
        (self.bits()[byte_offset] & (1 << bit_shift)) != 0
    }

    pub fn set(&mut self, bit_offset: usize) {
        let byte_offset = bit_offset / 8;
        let bit_shift = bit_offset % 8;
        self.bits_mut()[byte_offset] |= 1 << bit_shift;
    }

    pub fn clear(&mut self) {
        for byte in self.bits_mut().iter_mut() {
            *byte = 0;
        }
    }

    pub fn set_all(&mut self) {
        for byte in self.bits_mut().iter_mut() {
            *byte = !0;
        }
    }

    pub fn any(&self) -> bool {
        self.bits().iter().any(|&byte| byte != 0)
    }

    pub fn len_bits(&self) -> u64 {
        u64::try_from(self.bits().len())
            .unwrap()
            .checked_mul(8)
            .unwrap()
    }

    #[doc(hidden)]
    pub fn realloc_large_heap_allocated_objects(mut self, f: fn(Vec<u8>) -> Vec<u8>) -> Self {
        let previous_len = self.header_and_bits.len();
        self.header_and_bits = f(self.header_and_bits);
        assert_eq!(previous_len, self.header_and_bits.len());
        assert_eq!(Self::get_version(self.header()), VERSION);
        self
    }
}
