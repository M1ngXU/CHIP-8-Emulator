use std::collections::{ HashSet, LinkedList };
use std::fmt::{ Display, Debug };
use std::fmt;
use std::ops::{ Index, IndexMut };
use crate::output::Output;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Byte {
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

	pub fn from_usize(d: usize) -> Self {
		Self {
			data: d as u8
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

	pub fn add_byte(&self, byte: Byte) -> Byte {
		Byte::from(((self.data as u16 + byte.data as u16) & 0xFF) as u8)
	}
	
	pub fn decrease(&mut self, amount: u8) {
		self.data -= amount;
	}
}


#[derive(Eq, PartialEq, Copy, Clone)]
pub struct TwoBytes {
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

	pub fn as_usize(&self) -> usize {
		self.data as usize
	}

	pub fn as_byte(&self) -> Byte {
		Byte::from(self.data as u8)
	}

	pub fn mask_bits(&self, mask: &Self) -> Self {
		Self::from(self.data & mask.data)
	}

	pub fn shift_right(&self, amount: u8) -> Self {
		let mut mask = 0x00;
		for b in amount..=15 {
			mask |= 1 << b;
		}
		Self::from((self.data & mask) >> amount)
	}

	pub fn increase(&mut self, amount: u16) {
		self.data += amount;
	}

	pub fn increase_with_byte(&mut self, byte: Byte) {
		self.data += byte.data as u16;
	}

	pub fn add_byte(&self, byte: Byte) -> Self {
		Self::from(self.data + byte.data as u16)
	}

	pub fn add(&self, amount: u16) -> Self {
		Self::from(self.data + amount)
	}
}


#[derive(Debug)]
pub struct Memory {
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

	pub fn get_two_bytes(&self, start: TwoBytes) -> TwoBytes {
		TwoBytes::from(((self[start].data as u16) << 8) + self[start.data as usize + 1].data as u16)
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
	
	pub fn set_f(&mut self, v: bool) {
		self.data[0xF] = if v { Byte::from(1) } else { Byte::from(0) };
	}

	pub fn get_0(&self) -> Byte {
		self.data[0x0]
	}

	pub fn get_x(&mut self, v: TwoBytes) -> Byte {
		self.data[v.shift_right(8).mask_bits(&TwoBytes::from(0xF)).as_usize()]
	}

	pub fn set_x(&mut self, v: TwoBytes, value: Byte) {
		self.data[v.shift_right(8).mask_bits(&TwoBytes::from(0xF)).as_usize()] = value;
	}

	pub fn get_y(&mut self, v: TwoBytes) -> Byte {
		self.data[v.shift_right(4).mask_bits(&TwoBytes::from(0xF)).as_usize()]
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


pub struct State {
	memory: Memory,
	data_registers: DataRegisters,
	adress_register: TwoBytes,
	stack: LinkedList<TwoBytes>,
	pc: TwoBytes,
	pub output: Output,
	delay_timer: Byte,
	sound_timer: Byte,
	random_numbers: LinkedList<Byte>,
	awaiting_key: Option<TwoBytes>
}
impl State {
	pub fn new_chip8() -> Self {
		Self {
			memory: Memory::new(4096),
			data_registers: DataRegisters::new(16),
			adress_register: TwoBytes::new(),
			stack: LinkedList::new(),
			pc: TwoBytes::from(0x200),
			output: Output::new(64, 32, 25),
			delay_timer: Byte::new(),
			sound_timer: Byte::new(),
			random_numbers: {
				let start = std::time::SystemTime::now();
				let mut a = HashSet::new();
				while a.len() < 256 {
					let e = start.elapsed().unwrap();
					a.insert(((e.as_nanos() / e.as_millis().max(1)) % 256) as u8);
				}
				LinkedList::from_iter(a.drain().map(| n | Byte::from(n)))
			}, awaiting_key: None
		}
	}

	pub fn get_next_random(&mut self) -> Byte {
		let r = self.random_numbers.pop_front().unwrap();
		self.random_numbers.push_back(r);
		r
	}

	pub fn next_frame(&mut self) {
		if self.delay_timer.data > 0 {
			self.delay_timer.decrease(1);
		}
		if self.sound_timer.data > 0 {
			self.sound_timer.decrease(1);
			self.output.buzz();
			if self.sound_timer.data == 0 {
				self.output.stop_buzz();
			}
		}
	}

	pub fn load_memory(&mut self, bytes: Vec<u8>, starting_address: u16) {
		for (i, b) in bytes.into_iter().enumerate() {
			self.memory[i + starting_address as usize] = Byte::from(b);
		}
	}

	pub fn shutdown(&self) {
	}

	pub fn interpret_next(&mut self) {
		if let Some(k) = self.awaiting_key {
			if let Some(c) = self.output.get_current_input() {
				self.data_registers.set_x(k, Byte::from(c));
				self.awaiting_key = None;
			}
			return;
		}
		let current = self.memory.get_two_bytes(self.pc);
		let x = self.data_registers.get_x(current);
		let y = self.data_registers.get_y(current);
		let l3_const = current.mask_bits(&TwoBytes::from(0x0FFF));
		let l2_const = current.mask_bits(&TwoBytes::from(0x00FF)).as_byte();
		let l1_const = current.mask_bits(&TwoBytes::from(0x000F)).as_byte();
		match current.shift_right(12).as_usize() {
			0x0 | 0x1 | 0x2 | 0xB => {},
			_ => self.pc.increase(2)
		};
		match current.shift_right(12).as_usize() {
			0x0 => match l3_const.data {
				0x0E0 => {
					self.output.clear();
					self.pc.increase(2);
				}, 0x0EE => self.pc = self.stack.pop_back().unwrap(),
				_ => panic!("Can't handle rom execution.")
			}, 0x1 => {
				self.pc = l3_const;
			}, 0x2 => {
				self.stack.push_back(self.pc.add(2));
				self.pc = l3_const;
			}, 0x3 => if x == l2_const {
				self.pc.increase(2)
			}, 0x4 => if x != l2_const {
				self.pc.increase(2);
			}, 0x5 => if x == y {
				self.pc.increase(2);
			}, 0x6 => self.data_registers.set_x(current, l2_const),
			0x7 => self.data_registers.set_x(current, x.add_byte(l2_const)),
			0x8 => match l1_const.data {
				0x0 => self.data_registers.set_x(current, y),
				0x1 => self.data_registers.set_x(current, Byte::from(x.data | y.data)),
				0x2 => self.data_registers.set_x(current, x.mask_bits(&y)),
				0x3 => self.data_registers.set_x(current, Byte::from(x.data ^ y.data)),
				0x4 => {
					self.data_registers.set_f(false);
					if x.data > (0xFF - y.data) {
						self.data_registers.set_f(true);
					}
					self.data_registers.set_x(current, x.add_byte(y));
				}, 0x5 => {
					self.data_registers.set_f(true);
					if x.data < y.data {
						self.data_registers.set_f(false);
						self.data_registers.set_x(current, Byte::from(0xFF - y.data + x.data + 0x01));
					} else {
						self.data_registers.set_x(current, Byte::from(x.data - y.data));
					}
				}, 0x6 => {
					self.data_registers.set_f(x.get_bit(0));
					self.data_registers.set_x(current, x.shift_right(1));
				}, 0x7 => {
					self.data_registers.set_f(true);
					if y.data < x.data {
						self.data_registers.set_f(false);
						self.data_registers.set_x(current, Byte::from(0xFF - x.data + y.data + 0x01));
					} else {
						self.data_registers.set_x(current, Byte::from(y.data - x.data));
					}
				}, 0xE => {
					self.data_registers.set_f(x.get_bit(7));
					self.data_registers.set_x(current, x.shift_left(1));
				},
				_ => panic!("Unknown Variable Operation {}.", current)
			}, 0x9 => if x.data != y.data {
				self.pc.increase(2);
			}, 0xA => self.adress_register = l3_const,
			0xB => self.pc = l3_const.add_byte(self.data_registers.get_0()),
			0xC => {
				let masked_random_number = self.get_next_random().mask_bits(&l2_const);
				self.data_registers.set_x(current, masked_random_number)
			}, 0xD => {
				self.data_registers.set_f(false);
				for h in 0..current.mask_bits(&TwoBytes::from(0x000F)).data {
					for i in 0..8 {
						if self.memory[self.adress_register.add(h)].mask_bits(&Byte::from(1).shift_left(7 - i)).shift_right(7 - i).data == 1 {
							let screen_x = x.as_usize() + i as usize;
							let screen_y = y.as_usize() + h as usize;
							if self.output.get(screen_x as u32, screen_y as u32) {
								self.data_registers.set_f(true);
							}
							self.output.swap(screen_x as u32, screen_y as u32);
						}
					}
				}
			}, 0xE => match l2_const.data {
				0x9E => if self.output.is_pressed(x.data) {
					self.pc.increase(2);
				}, 0xA1 => if !self.output.is_pressed(x.data) {
					self.pc.increase(2);
				}, _ => unreachable!()
			}, 0xF => {
				match l2_const.data as u8 {
					0x07 => self.data_registers.set_x(current, self.delay_timer),
					0x0A => {
						log::info!("Awaiting a key (hex) input ...");
						self.awaiting_key = Some(current);
					}, 0x15 => self.delay_timer = x,
					0x18 => {
						if x.data == 0 {
							self.output.stop_buzz();
						}
						self.sound_timer = x;
					},
					0x1E => self.adress_register.increase_with_byte(x),
					0x29 => self.adress_register = TwoBytes::from(x.data as u16 * 5),
					0x33 => {
						self.memory[self.adress_register.as_usize()] = Byte::from_usize(x.as_usize() / 100);
						self.memory[self.adress_register.as_usize() + 1] = Byte::from_usize(x.as_usize() % 100 / 10);
						self.memory[self.adress_register.as_usize() + 2] = Byte::from_usize(x.as_usize() % 10);
					}, 0x55 => for i in 0..=l3_const.shift_right(8).as_usize() {
						self.memory[self.adress_register.add(i as u16)] = self.data_registers[i];
					}, 0x65 => for i in 0..=l3_const.shift_right(8).as_usize() {
						self.data_registers[i] = self.memory[self.adress_register.add(i as u16)];
					}, _ => panic!("Unknown memory opcode {}.", current.data)
				}
			}, _ => panic!("Unknown opcode {}", current)
		}
	}
}

#[test]
fn save_load_registers() {
	let mut cut = State::new_chip8();
	cut.load_memory(vec![
		0x60, 0x10,
		0xA0, 0x00,
		0xF0, 0x55,
		0x60, 0x01,
		0xF0, 0x65
	], 0x200);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.memory[0].data, 0x10);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[0].data, 0x10);
}

#[test]
fn goto() {
	let mut cut = State::new_chip8();
	cut.load_memory(vec![ 0x10, 0x11 ], 0x200);
	assert_eq!(cut.pc.data, 0x200);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x011);
}

#[test]
fn subroutine() {
	let mut cut = State::new_chip8();
	// call subroutine, then return
	cut.load_memory(vec![ 0x22, 0x04, 0x00, 0x00, 0x00, 0xEE ], 0x200);
	assert_eq!(cut.pc.data, 0x200);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x204);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x202);
}

#[test]
fn compare_data_registers() {
	let mut cut = State::new_chip8();
	// set vx, vy, then compare
	cut.load_memory(vec![ 0x60, 0x01, 0x61, 0x01, 0x30, 0x01, 0x00, 0x00, 0x41, 0x00, 0x00, 0x00, 0x50, 0x10, 0x00, 0x00, 0x90, 0x10 ], 0x200);
	assert_eq!(cut.data_registers[0].data, 0x00);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[0].data, 0x01);
	assert_eq!(cut.data_registers[1].data, 0x00);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x01);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x208);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x20c);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x210);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x212);
}

#[test]
fn data_register_operations() {
	let mut cut = State::new_chip8();
	// set vx, vy, then operate
	cut.load_memory(vec![
		0x60, 0x05, // 0 = 5
		0x70, 0x01, // 0 += 1
		0x61, 0x01, // 1 = 1
		0x82, 0x00, // 2 = v0
		0x81, 0x01, // 1 |= v0
		0x81, 0x03, // 1 ^= v0
		0x61, 0x07, // 1 = 1
		0x81, 0x02, // 1 &= v0
		0x81, 0x04, // 1 += v0
		0x61, 0xFF, // 1 = FF
		0x81, 0x04, // 1 += v0
		0x60, 0x02, // 0 = 2
		0x81, 0x05, // 1 -= v0
		0x61, 0x01, // 1 = 1
		0x81, 0x05, // 1 -= v0
		0x60, 0x01, // 0 = 1
		0x61, 0xFF, // 0 = 1
		0x80, 0x06, // 0 >>= 1
		0x81, 0x0E, // 1 <<= 1
		0x60, 0x01, // 0 = 1
		0x80, 0x17, // 0 = v1 - v0
	], 0x200);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[0].data, 0x05);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[0].data, 0x06);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x01);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[2].data, 0x06);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x07);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x01);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x06);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x0c);
	assert_eq!(cut.data_registers[0xF].data, 0x00);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x05);
	assert_eq!(cut.data_registers[0xF].data, 0x01);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0x03);
	assert_eq!(cut.data_registers[0xF].data, 0x01);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0xFF);
	assert_eq!(cut.data_registers[0xF].data, 0x00);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[0].data, 0x00);
	assert_eq!(cut.data_registers[0xF].data, 0x01);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[1].data, 0b1111_1110);
	assert_eq!(cut.data_registers[0xF].data, 0x01);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[0].data, 0b1111_1101);
	assert_eq!(cut.data_registers[0xF].data, 0x01);
}

#[test]
fn set_adress_register() {
	let mut cut = State::new_chip8();
	cut.load_memory(vec![ 0xA1, 0x23 ], 0x200);
	assert!(!cut.interpret_next());
	assert_eq!(cut.adress_register.data, 0x123);
}

#[test]
fn jump_to() {
	let mut cut = State::new_chip8();
	cut.load_memory(vec![
		0x12, 0x04,
		0x00, 0x00,
		0x60, 0x12,
		0xB1, 0x23
	], 0x200);
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x204);
	assert!(!cut.interpret_next());
	assert!(!cut.interpret_next());
	assert_eq!(cut.pc.data, 0x135);
}

#[test]
fn random_numbers() {
	let mut cut = State::new_chip8();
	cut.random_numbers = LinkedList::new();
	cut.random_numbers.push_back(Byte::from(0b0011_0000));
	cut.load_memory(vec![
		0xC0, 0b0001_0010
	], 0x200);
	assert!(!cut.interpret_next());
	assert_eq!(cut.data_registers[0].data, 0b0001_0000);
}

#[test]
fn new_state() {
	let cut = State::new_chip8();
	assert_eq!(cut.memory[TwoBytes::from(4095)], Byte::new());
	assert_eq!(cut.data_registers[Byte::new()], Byte::new());
	assert_eq!(cut.adress_register, TwoBytes::new());
	assert_eq!(cut.stack.len(), 0);
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

	assert_eq!(Byte::from(0b1111_1111).add(0b0000_0010), Byte::from(0b0000_0001));
	assert_eq!(Byte::from(0b1111_1111).add_byte(Byte::from(0b0000_0010)), Byte::from(0b0000_0001));
}

#[test]
fn two_bytes() {
	let cut = TwoBytes::from(0b0000_1110_0000_0000);
	assert_eq!(cut.get_bit(8), false);
	assert_eq!(cut.get_bit(9), true);
	assert_eq!(cut.get_bit(10), true);
	assert_eq!(cut.get_bit(11), true);
	assert_eq!(cut.get_bit(12), false);
	assert_eq!(cut.get_bit(13), false);
	assert_eq!(cut.get_bit(14), false);
	assert_eq!(cut.get_bit(15), false);

	assert_eq!(cut.as_usize(), 3584);

	assert_eq!(cut.shift_left(4), TwoBytes::from(0b1110_0000_0000_0000));
	assert_eq!(cut.shift_right(4), TwoBytes::from(0b0000_0000_1110_0000));
}