use std::collections::HashMap;

#[rustfmt::skip]
const VARICODE: [u32; 128] = [
    // 0x00
    0b1010101011, 0b1011011011, 0b1011101101, 0b1101110111,
    0b1011101011, 0b1101011111, 0b1011101111, 0b1011111101,
    0b1011111111, 0b11101111, 0b11101, 0b1101101111,
    0b1011011101, 0b11111, 0b1101110101, 0b1110101011,
    // 0x10
    0b1011110111, 0b1011110101, 0b1110101101, 0b1110101111,
    0b1101011011, 0b1101101011, 0b1101101101, 0b1101010111,
    0b1101111011, 0b1101111101, 0b1110110111, 0b1101010101,
    0b1101011101, 0b1110111011, 0b1011111011, 0b1101111111,
    // 0x20
    0b1, 0b111111111, 0b101011111, 0b111110101,
    0b111011011, 0b1011010101, 0b1010111011, 0b101111111,
    0b11111011, 0b11110111, 0b101101111, 0b111011111,
    0b1110101, 0b110101, 0b1010111, 0b110101111,
    // 0x30
    0b10110111, 0b10111101, 0b11101101, 0b11111111,
    0b101110111, 0b101011011, 0b101101011, 0b110101101,
    0b110101011, 0b110110111, 0b11110101, 0b110111101,
    0b111101101, 0b1010101, 0b111010111, 0b1010101111,
    // 0x40
    0b1010111101, 0b1111101, 0b11101011, 0b10101101,
    0b10110101, 0b1110111, 0b11011011, 0b11111101,
    0b101010101, 0b1111111, 0b111111101, 0b101111101,
    0b11010111, 0b10111011, 0b11011101, 0b10101011,
    // 0x50
    0b11010101, 0b111011101, 0b10101111, 0b1101111,
    0b1101101, 0b101010111, 0b110110101, 0b101011101,
    0b101110101, 0b101111011, 0b1010101101, 0b111110111,
    0b111101111, 0b111111011, 0b1010111111, 0b101101101,
    // 0x60
    0b1011011111, 0b1011, 0b1011111, 0b101111,
    0b101101, 0b11, 0b111101, 0b1011011,
    0b101011, 0b1101, 0b111101011, 0b10111111,
    0b11011, 0b111011, 0b1111, 0b111,
    // 0x70
    0b111111, 0b110111111, 0b10101, 0b10111,
    0b101, 0b110111, 0b1111011, 0b1101011,
    0b11011111, 0b1011101, 0b111010101, 0b1010110111,
    0b110111011, 0b1010110101, 0b1011010111, 0b1110110101,
];

fn bits(x: u32) -> impl Iterator<Item = bool> + Clone {
    // Extract bits, starting from the most significant 1-bit.
    (0..=x.ilog2()).rev().map(move |i| ((x >> i) & 1) != 0)
}

/// Encode an ASCII byte into a stream of bits.
///
/// # Panics
///
/// Panics if the byte is not valid ASCII (outside of the range 0-127).
pub fn encode_ascii_byte(ascii: u8) -> impl Iterator<Item = bool> + Clone {
    bits(VARICODE[ascii as usize] << 2)
}

pub struct VaricodeDecode {
    bits: u32,
    lookup: HashMap<u32, u8>,
}

impl VaricodeDecode {
    pub fn new() -> Self {
        Self {
            bits: 0,
            lookup: VARICODE
                .iter()
                .enumerate()
                .map(|(i, &x)| (x << 2, i as u8))
                .collect(),
        }
    }

    /// Process an incoming bit.
    ///
    /// If a byte has been successfully decoded from the sequence of bits,
    /// it will be returned.
    pub fn process(&mut self, bit: bool) -> Option<u8> {
        self.bits = (self.bits << 1) | bit as u32;
        let lookup = self.lookup.get(&self.bits).copied();
        if (self.bits & 0b11) == 0b00 {
            self.bits = 0;
        }
        lookup
    }
}
