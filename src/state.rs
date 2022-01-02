use std::collections::LinkedList;
use std::fmt::{ Display, Debug };
use std::fmt;
use std::ops::{ Index, IndexMut };

#[derive(Eq, PartialEq)]
struct Byte {
	data: u8
}
impl Display for Byte {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "0x{:x}", self.data)
	}
}
impl Debug for Byte {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "0b{:b}", self.data)
	}
}
impl Byte {
	pub fn new() -> Self {
		Self {
			data: 0
		}
	}

	pub fn from(d: u8) -> Self {
		Self {
			data: d
		}
	}
	
	pub fn as_usize(&self) -> usize {
		self.data as usize
	}

	pub fn mask_bits(&self, mask: &Self) -> Self {
		Self::from(self.data & mask.data)
	}

	pub fn shift_left(&self, amount: u8) -> Self {
		let mut mask = 0xFF;
		for b in 0..amount {
			mask = mask & !(1 << (7 - b));
		}
		Self::from((self.data & mask) << amount)
	}

	pub fn shift_right(&self, amount: u8) -> Self {
		let mut mask = 0xFF;
		for b in 0..amount {
			mask = mask & !(1 << b);
		}
		Self::from((self.data & mask) >> amount)
	}

	pub fn get_bit(&self, bit: u8) -> bool {
		self.mask_bits(&Byte::from(1).shift_left(bit)).shift_right(bit).data == 1
	}
}


#[derive(Eq, PartialEq)]
struct TwoBytes {
	data: u16
}
impl Display for TwoBytes {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "0x{:x}", self.data)
	}
}
impl Debug for TwoBytes {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "0b{:b}", self.data)
	}
}
impl TwoBytes {
	pub fn new() -> Self {
		Self {
			data: 0
		}
	}

	pub fn from(d: u16) -> Self {
		Self {
			data: d
		}
	}

	pub fn as_usize(self) -> usize {
		self.data as usize
	}

	pub fn mask_bits(&self, mask: &Self) -> Self {
		Self::from(self.data & mask.data)
	}

	pub fn shift_left(&self, amount: u8) -> Self {
		let mut mask = 0xFF;
		for b in 0..amount {
			mask = mask & !(1 << (7 - b));
		}
		Self::from((self.data & mask) << amount)
	}

	pub fn shift_right(&self, amount: u8) -> Self {
		let mut mask = 0xFF;
		for b in 0..amount {
			mask = mask & !(1 << b);
		}
		Self::from((self.data & mask) >> amount)
	}

	pub fn get_bit(&self, bit: u8) -> bool {
		self.mask_bits(&Byte::from(1).shift_left(bit)).shift_right(bit).data == 1
	}
}


#[derive(Debug)]
struct Memory {
	data: Vec<Byte>
}
impl Memory {
	pub fn new(size: u64) -> Self {
		let mut new = Self {
			data: Vec::new()
		};
		for _ in 0..size {
			new.data.push(Byte::new());
		}
		new
	}
}
impl Index<TwoBytes> for Memory {
	type Output = Byte;

	fn index(&self, i: TwoBytes) -> &Self::Output {
		&self[i.as_usize()]
	}
}
impl Index<usize> for Memory {
	type Output = Byte;

	fn index(&self, i: usize) -> &Self::Output {
		&self.data[i]
	}
}
impl IndexMut<TwoBytes> for Memory {
	fn index_mut(&mut self, i: TwoBytes) -> &mut Self::Output {
		&mut self.data[i.as_usize()]
	}
}
impl IndexMut<usize> for Memory {
	fn index_mut(&mut self, i: usize) -> &mut Self::Output {
		&mut self.data[i]
	}
}


struct DataRegisters {
	data: Vec<Byte>
}
impl DataRegisters {
	pub fn new(size: u64) -> Self {
		let mut new = Self {
			data: Vec::new()
		};
		for _ in 0..size {
			new.data.push(Byte::new());
		}
		new
	}
}
impl Index<Byte> for DataRegisters {
	type Output = Byte;

	fn index(&self, i: Byte) -> &Self::Output {
		&self[i.as_usize()]
	}
}
impl Index<usize> for DataRegisters {
	type Output = Byte;

	fn index(&self, i: usize) -> &Self::Output {
		&self.data[i]
	}
}
impl IndexMut<Byte> for DataRegisters {
	fn index_mut(&mut self, i: Byte) -> &mut Self::Output {
		&mut self.data[i.as_usize()]
	}
}
impl IndexMut<usize> for DataRegisters {
	fn index_mut(&mut self, i: usize) -> &mut Self::Output {
		&mut self.data[i]
	}
}


struct AdressRegister {
	data: TwoBytes
}
impl AdressRegister {
	pub fn new() -> Self {
		Self {
			data: TwoBytes::new()
		}
	}
}


struct Stack {
	data: LinkedList<TwoBytes>
}
impl Stack {
	pub fn new() -> Self {
		Self {
			data: LinkedList::new()
		}
	}
}


struct State {
	memory: Memory,
	data_registers: DataRegisters,
	adress_register: AdressRegister,
	stack: Stack
}
impl State {
	pub fn new_chip8() -> Self {
		let mut new = Self {
			memory: Memory::new(4096),
			data_registers: DataRegisters::new(16),
			adress_register: AdressRegister::new(),
			stack: Stack::new()
		};
		for (i, b) in [
			0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
			0x20, 0x60, 0x20, 0x20, 0x70, // 1
			0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
			0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
			0x90, 0x90, 0xF0, 0x10, 0x10, // 4
			0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
			0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
			0xF0, 0x10, 0x20, 0x40, 0x40, // 7
			0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
			0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
			0xF0, 0x90, 0xF0, 0x90, 0x90, // A
			0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
			0xF0, 0x80, 0x80, 0x80, 0xF0, // C
			0xE0, 0x90, 0x90, 0x90, 0xE0, // D
			0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
			0xF0, 0x80, 0xF0, 0x80, 0x80  // F
		].into_iter().enumerate() {
			new.memory[i] = Byte::from(b);
		}
		new
	}
}

#[test]
fn new_state() {
	let cut = State::new_chip8();
	assert_eq!(cut.memory[TwoBytes::from(4095)], Byte::new());
	assert_eq!(cut.data_registers[Byte::new()], Byte::new());
	assert_eq!(cut.adress_register.data, TwoBytes::new());
	assert_eq!(cut.stack.data.len(), 0);
}

#[test]
fn memory_indexing() {
	let mut cut = Memory::new(2);
	assert_eq!(cut[1], Byte::from(0));
	cut[1] = Byte::from(2);
	assert_eq!(cut[1], Byte::from(2));
	assert_eq!(cut.data[1], Byte::from(2));
}

#[test]
fn data_registers_indexing() {
	let mut cut = DataRegisters::new(2);
	assert_eq!(cut[1], Byte::from(0));
	cut[1] = Byte::from(2);
	assert_eq!(cut[1], Byte::from(2));
	assert_eq!(cut.data[1], Byte::from(2));
}

#[test]
fn byte() {
	let cut = Byte::from(0b0001_0100);
	assert_eq!(cut.get_bit(0), false);
	assert_eq!(cut.get_bit(1), false);
	assert_eq!(cut.get_bit(2), true);
	assert_eq!(cut.get_bit(3), false);
	assert_eq!(cut.get_bit(4), true);
	assert_eq!(cut.get_bit(5), false);
	assert_eq!(cut.get_bit(6), false);
	assert_eq!(cut.get_bit(7), false);

	assert_eq!(cut.as_usize(), 20);

	assert_eq!(cut.shift_left(4), Byte::from(0b0100_0000));
	assert_eq!(cut.shift_right(4), Byte::from(0b0000_0001));
}

#[test]
fn two_bytes() {
	let cut = TwoBytes::from(0b0000_1110_0000_0000);
	assert_eq!(cut.get_bit(8), true);
	assert_eq!(cut.get_bit(9), true);
	assert_eq!(cut.get_bit(10), true);
	assert_eq!(cut.get_bit(11), false);
	assert_eq!(cut.get_bit(12), false);
	assert_eq!(cut.get_bit(13), false);
	assert_eq!(cut.get_bit(14), false);
	assert_eq!(cut.get_bit(15), false);

	assert_eq!(cut.as_usize(), 20);

	assert_eq!(cut.shift_left(4), Byte::from(0b1110_0000_0000_0000));
	assert_eq!(cut.shift_right(4), Byte::from(0b0000_0000_1110_0000));
}