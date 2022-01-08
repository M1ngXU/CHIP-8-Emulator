use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

/// B is the amount of bits and isn't allowed to exceed 31
/// max is the biggest possible number, always being full of `1`s
#[derive(Debug, Clone, Copy)]
pub struct FixedBitNumber<const N: u8> {
    number: u32
}
impl<const A: u8> FixedBitNumber<A> {
    pub fn new() -> Self {
        Self::from(0)
    }

    /// creates a new `FixedBitNumber` by a u32, 'overflowing' if the number exceeds the max-size
    pub fn from(number: u32) -> Self {
        Self {
            number: number & Self::get_max()
        }
    }

    pub fn from_u8(number: u8) -> Self {
        Self::from(number.into())
    }

    pub fn len(&self) -> u8 {
        A
    }

    pub fn from_combined<const B: u8, const C: u8>(n1: &FixedBitNumber<B>, n2: &FixedBitNumber<C>) -> Self {
        Self::from(((n1.number & Self::get_max_by_length(A - C)) << C) + (n2.number & Self::get_max()))
    }

    pub fn get_max() -> u32 {
        Self::get_max_by_length(A)
    }

    pub fn get_max_by_length(length: u8) -> u32 {
        (1 << length.min(32 - 1)) - 1
    }

    pub fn get_bitrange(&self, start: u8, length: u8) -> Self {
        let mut mask = 0;
        // shouldn't overflow
        for i in start..(start + length.min(31 - start)) {
            mask |= 1 << i;
        }
        Self::from((self.number & mask) >> start)
    }

    pub fn into_u8(self) -> u8 {
        (self.number & u8::MAX as u32) as u8
    }

    pub fn into_u32(self) -> u32 {
        self.number
    }

    pub fn into_usize(self) -> usize {
        self.number as usize
    }

    pub fn get_bit(&self, bit: u8) -> bool {
        bit <= A && ((self.number & (1 << bit)) >> bit) == 1
    }

    /// increases (a + b), returns true if there was a overflow
    pub fn increase<const B: u8>(&mut self, other: &FixedBitNumber<B>) -> bool {
        self.increase_by_u32(other.number)
    }

    pub fn increase_by_u32(&mut self, other: u32) -> bool {
        let mut overflowed = false;
        let new = self.number + other;
        if new > Self::get_max() {
            overflowed = true;
        }
        self.set_by_u32(new);
        overflowed
    }

    pub fn add_by_u32(&self, value: u32) -> Self {
        Self::from(self.number + value)
    }

    pub fn add_by_u8(&self, value: u8) -> Self {
        Self::from(self.number + value as u32)
    }

    pub fn add_by_usize(&self, value: usize) -> Self {
        self.add_by_u32(value as u32)
    }

    pub fn add<const B: u8>(&self, other: &FixedBitNumber<B>) -> Self {
        self.add_by_u32(other.number)
    }

    pub fn set_by_u32(&mut self, value: u32) {
        self.number = value & Self::get_max();
    }

    pub fn set<const B: u8>(&mut self, other: &FixedBitNumber<B>) {
        self.number = other.number & Self::get_max();
    }

    /// Sets the value to 0 if false, 1 if true
    pub fn set_bool(&mut self, v: bool) {
        if v {
            self.set_by_u32(1);
        } else {
            self.set_by_u32(0);
        }
    }

    /// decreases (a - b), returns true if there was NO borrow
    pub fn decrease<const B: u8>(&mut self, other: &FixedBitNumber<B>) -> bool {
        self.decrease_by_u32(other.number)
    }

    pub fn decrease_by_u32(&mut self, other: u32) -> bool {
        let borrow = other > self.number;
        self.set_by_u32(self.number + Self::get_max() - (other & Self::get_max()) + 1);
        !borrow
    }

    /// reversed decrease: (a = b - a), returns true if there was NO borrow
    pub fn reversed_decrease<const B: u8>(&mut self, other: &FixedBitNumber<B>) -> bool {
        let borrow = self.number > other.number;
        self.set_by_u32((other.number & Self::get_max()) + Self::get_max() - self.number + 1);
        !borrow
    }

    pub fn shift_right(&mut self) -> bool {
        let remaining = self.number & (Self::get_max() - 1);
        let shifted_one = remaining != self.number;
        self.set_by_u32(remaining >> 1);
        shifted_one
    }

    pub fn shift_left(&mut self) -> bool {
        let remaining = self.number & ((1 << (A - 1)) - 1);
        let shifted_one = remaining != self.number;
        self.set_by_u32(remaining << 1);
        shifted_one
    }

    pub fn and<const B: u8>(&mut self, other: &FixedBitNumber<B>) {
        self.set_by_u32(self.number & other.number)
    }

    pub fn xor<const B: u8>(&mut self, other: &FixedBitNumber<B>) {
        self.set_by_u32(self.number ^ other.number)
    }

    pub fn or<const B: u8>(&mut self, other: &FixedBitNumber<B>) {
        self.set_by_u32(self.number | other.number)
    }

    pub fn execute_if_equals<const B: u8, F: FnOnce()>(&self, other: &FixedBitNumber<B>, f: F) {
        if self.number == other.number {
            f();
        }
    }

    pub fn execute_if_not_equals<const B: u8, F: FnOnce()>(&self, other: &FixedBitNumber<B>, f: F) {
        if self.number != other.number {
            f();
        }
    }
}

impl<const A: u8, const B: u8> Index<&FixedBitNumber<A>> for Vec<FixedBitNumber<B>> {
    type Output = FixedBitNumber<B>;

    fn index(&self, index: &FixedBitNumber<A>) -> &Self::Output {
        &self[index.into_usize()]
    }
}
impl<const A: u8, const B: u8> IndexMut<&FixedBitNumber<A>> for Vec<FixedBitNumber<B>> {
    fn index_mut(&mut self, index: &FixedBitNumber<A>) -> &mut Self::Output {
        &mut self[index.into_usize()]
    }
}
impl<const A: u8> Display for FixedBitNumber<A> {
    fn fmt(&self, f: &mut Formatter<>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.number)
    }
}
impl<const A: u8> PartialEq for FixedBitNumber<A> {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}
impl<const A: u8> Eq for FixedBitNumber<A> {}
impl<const A: u8> Default for FixedBitNumber<A> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait IntoEmpty {
    fn into_empty(self) where Self: Sized {}
}
impl<T> IntoEmpty for T {}

/*
#[cfg(test)]
mod tests {
    use crate::fixed_bit_numbers::FixedBitNumber;

    #[test]
    fn new_number() {
        assert_eq!(FixedBitNumber::new(2).into_u32(), 0);
        assert_eq!(FixedBitNumber::from(3, 2).into_u32(), 3);
        assert_eq!(FixedBitNumber::from(4, 2).into_u32(), 0);
    }

    #[test]
    fn get_bit() {
        let cut = FixedBitNumber::from(1, 2);
        assert!(cut.get_bit(0));
        assert!(!cut.get_bit(1));
        assert!(!cut.get_bit(2));
    }

    #[test]
    fn increase() {
        let mut cut = FixedBitNumber::new(2);
        assert!(!cut.increase(&FixedBitNumber::from(2, 2)));
        assert!(cut.increase(&FixedBitNumber::from(0b1111_1101, 8)));
        assert!(cut.increase(&FixedBitNumber::from(1, 1)));
        assert_eq!(cut.into_u32(), 0);
    }

    #[test]
    fn set() {
        let mut cut = FixedBitNumber::new(2);
        cut.set_by_u32(15);
        assert_eq!(cut.into_u32(), 3);
        cut.set_bool(true);
        assert_eq!(cut.into_u32(), 1);
        cut.set_bool(false);
        assert_eq!(cut.into_u32(), 0);
    }

    #[test]
    fn decrease() {
        let mut cut = FixedBitNumber::from(1, 2);
        assert!(cut.decrease(&FixedBitNumber::from(1, 3)));
        assert_eq!(cut.into_u32(), 0);
        assert!(!cut.decrease(&FixedBitNumber::from(1, 1)));
        assert_eq!(cut.into_u32(), 3);
        cut.set_by_u32(1);
        assert!(cut.reversed_decrease(&FixedBitNumber::from(3, 2)));
        assert!(!cut.reversed_decrease(&FixedBitNumber::new(2)));
        assert_eq!(cut.into_u32(), 2);
    }

    #[test]
    fn shift() {
        let mut cut = FixedBitNumber::from(0b111, 3);
        assert!(cut.shift_right());
        assert_eq!(cut.into_u32(), 0b011);
        assert!(!cut.shift_left());
        assert_eq!(cut.into_u32(), 0b110);
        cut = FixedBitNumber::from(0b111, 3);
        assert!(cut.shift_left());
        assert_eq!(cut.into_u32(), 0b110);
    }

    #[test]
    fn logical_ops() {
        let mut cut1 = FixedBitNumber::from(0b00_110, 3);
        let cut2 = FixedBitNumber::from(0b10_101, 5);
        cut1.and(&cut2);
        assert_eq!(cut1.into_u32(), 0b100);
        cut1.xor(&cut2);
        assert_eq!(cut1.into_u32(), 0b001);
        cut1.or(&cut2);
        assert_eq!(cut1.into_u32(), 0b101)
    }
}*/