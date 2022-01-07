use std::ops::{Index, IndexMut};

/// B is the amount of bits and isn't allowed to exceed 31
/// max is the biggest possible number, always being full of `1`s
#[derive(Eq, PartialEq, Debug)]
pub struct FixedBitNumber<const B: u8> {
    number: u32,
    max: u32
}
impl<const B: u8> FixedBitNumber<B> {
    pub fn new() -> Self {
        Self::from(0)
    }

    /// creates a new `FixedBitNumber` by a u32, 'overflowing' if the number exceeds the max-size
    pub fn from(number: u32) -> Self {
        let max = (1 << B.min(32 - 1)) - 1;
        Self {
            number: number & max,
            max
        }
    }

    pub fn into_u32(&self) -> u32 {
        self.number
    }

    pub fn get_bit(&self, bit: u8) -> bool {
        bit <= B && ((self.number & (1 << bit)) >> bit) == 1
    }

    /// increases (a + b), returns true if there was a overflow
    pub fn increase<const M: u8>(&mut self, other: &FixedBitNumber<M>) -> bool {
        let mut overflow = false;
        let new = self.number + other.number;
        if new > self.max {
            overflow = true;
        }
        self.set(new);
        overflow
    }

    pub fn set(&mut self, value: u32) {
        self.number = value & self.max;
    }

    /// Sets the value to 0 if false, 1 if true
    pub fn set_bool(&mut self, v: bool) {
        if v {
            self.set(1);
        } else {
            self.set(0);
        }
    }

    /// decreases (a - b), returns true if there was NO borrow
    pub fn decrease<const M: u8>(&mut self, other: &FixedBitNumber<M>) -> bool {
        let borrow = other.number > self.number;
        self.set(self.number + self.max + 1 - other.number & self.max);
        !borrow
    }

    /// reversed decrease: (a = b - a), returns true if there was NO borrow
    pub fn reversed_decrease<const M: u8>(&mut self, other: &FixedBitNumber<M>) -> bool {
        let borrow = self.number > other.number;
        self.set(other.number + self.max + 1 - self.number);
        !borrow
    }

    pub fn shift_right(&mut self) -> bool {
        let remaining = self.number & (self.max - 1);
        let shifted_one = if remaining == self.number { false } else { true };
        self.set(remaining >> 1);
        shifted_one
    }

    pub fn shift_left(&mut self) -> bool {
        let remaining = self.number & ((1 << (B - 1)) - 1);
        let shifted_one = if remaining == self.number { false } else { true };
        self.set(remaining << 1);
        shifted_one
    }

    pub fn and<const M: u8>(&mut self, other: &FixedBitNumber<M>) {
        self.set(self.number & other.number)
    }

    pub fn xor<const M: u8>(&mut self, other: &FixedBitNumber<M>) {
        self.set(self.number ^ other.number)
    }

    pub fn or<const M: u8>(&mut self, other: &FixedBitNumber<M>) {
        self.set(self.number | other.number)
    }
}
impl<const N: u8> From<FixedBitNumber<N>> for u32 {
    fn from(n: FixedBitNumber<N>) -> Self {
        n.into_u32()
    }
}

impl<const N: u8> Index<FixedBitNumber<N>> for Vec<FixedBitNumber<N>> {
    type Output = FixedBitNumber<N>;

    fn index(&self, index: FixedBitNumber<N>) -> &Self::Output {
        &self[index.into_u32() as usize]
    }
}
impl<const N: u8> IndexMut<FixedBitNumber<N>> for Vec<FixedBitNumber<N>> {
    fn index_mut(&mut self, index: FixedBitNumber<N>) -> &mut Self::Output {
        &mut self[index.into_u32() as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::fixed_bit_numbers::FixedBitNumber;

    #[test]
    fn new_number() {
        assert_eq!(FixedBitNumber::<2>::new().into_u32(), 0);
        assert_eq!(FixedBitNumber::<2>::from(3).into_u32(), 3);
        assert_eq!(FixedBitNumber::<2>::from(4).into_u32(), 0);
    }

    #[test]
    fn get_bit() {
        let cut = FixedBitNumber::<2>::from(1);
        assert!(cut.get_bit(0));
        assert!(!cut.get_bit(1));
        assert!(!cut.get_bit(2));
    }

    #[test]
    fn increase() {
        let mut cut = FixedBitNumber::<2>::new();
        assert!(!cut.increase(&FixedBitNumber::<2>::from(2)));
        assert!(cut.increase(&FixedBitNumber::<8>::from(0b1111_1101)));
        assert!(cut.increase(&FixedBitNumber::<1>::from(1)));
        assert_eq!(cut.into_u32(), 0);
    }

    #[test]
    fn set() {
        let mut cut = FixedBitNumber::<2>::new();
        cut.set(15);
        assert_eq!(cut.into_u32(), 3);
        cut.set_bool(true);
        assert_eq!(cut.into_u32(), 1);
        cut.set_bool(false);
        assert_eq!(cut.into_u32(), 0);
    }

    #[test]
    fn decrease() {
        let mut cut = FixedBitNumber::<2>::from(1);
        assert!(cut.decrease(&FixedBitNumber::<3>::from(1)));
        assert_eq!(cut.into_u32(), 0);
        assert!(!cut.decrease(&FixedBitNumber::<1>::from(1)));
        assert_eq!(cut.into_u32(), 3);
        cut.set(1);
        assert!(cut.reversed_decrease(&FixedBitNumber::<2>::from(3)));
        assert!(!cut.reversed_decrease(&FixedBitNumber::<2>::new()));
        assert_eq!(cut.into_u32(), 2);
    }

    #[test]
    fn shift() {
        let mut cut = FixedBitNumber::<3>::from(0b111);
        assert!(cut.shift_right());
        assert_eq!(cut.into_u32(), 0b011);
        assert!(!cut.shift_left());
        assert_eq!(cut.into_u32(), 0b110);
        cut = FixedBitNumber::<3>::from(0b111);
        assert!(cut.shift_left());
        assert_eq!(cut.into_u32(), 0b110);
    }

    #[test]
    fn logical_ops() {
        let mut cut1 = FixedBitNumber::<3>::from(0b00_110);
        let cut2 = FixedBitNumber::<5>::from(0b10_101);
        cut1.and(&cut2);
        assert_eq!(cut1.into_u32(), 0b100);
        cut1.xor(&cut2);
        assert_eq!(cut1.into_u32(), 0b001);
        cut1.or(&cut2);
        assert_eq!(cut1.into_u32(), 0b101)
    }
}