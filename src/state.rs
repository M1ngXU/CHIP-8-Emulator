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
	
	pub fn as_usize(self) -> usize {
		self.data as usize
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


#[derive(Debug, Eq, PartialEq)]
struct DataRegister {
	data: Byte
}
impl DataRegister {
	pub fn new() -> Self {
		Self {
			data: Byte::new()
		}
	}
}


struct DataRegisters {
	data: Vec<DataRegister>
}
impl DataRegisters {
	pub fn new(size: u64) -> Self {
		let mut new = Self {
			data: Vec::new()
		};
		for _ in 0..size {
			new.data.push(DataRegister::new());
		}
		new
	}
}
impl Index<Byte> for DataRegisters {
	type Output = DataRegister;

	fn index(&self, i: Byte) -> &Self::Output {
		&self.data[i.as_usize()]
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
	assert_eq!(cut.data_registers[Byte::new()], DataRegister::new());
	assert_eq!(cut.adress_register.data, TwoBytes::new());
	assert_eq!(cut.stack.data.len(), 0);
}