use std::collections::LinkedList;
use std::fmt::{ Display, Debug };
use std::fmt;
use std::ops::{ Index, IndexMut };

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
		Byte::from(((self.data as u16 + 0xFF + byte.data as u16) & 0xFF) as u8)
	}

	pub fn add(&self, amount: u8) -> Byte {
		Byte::from(((self.data as u16 + 0xFF + amount as u16) & 0xFF) as u8)
	}
}


#[derive(Eq, PartialEq, Copy, Clone)]
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

	pub fn as_usize(&self) -> usize {
		self.data as usize
	}

	pub fn as_byte(&self) -> Byte {
		Byte::from(self.data as u8)
	}

	pub fn mask_bits(&self, mask: &Self) -> Self {
		Self::from(self.data & mask.data)
	}

	pub fn shift_left(&self, amount: u8) -> Self {
		let mut mask = 0;
		for b in 0..=(15 - amount) {
			mask |= 1 << b;
		}
		Self::from((self.data & mask) << amount)
	}

	pub fn shift_right(&self, amount: u8) -> Self {
		let mut mask = 0x00;
		for b in amount..=15 {
			mask |= 1 << b;
		}
		Self::from((self.data & mask) >> amount)
	}

	pub fn get_bit(&self, bit: u8) -> bool {
		self.mask_bits(&TwoBytes::from(1).shift_left(bit)).shift_right(bit).data == 1
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

	pub fn get_two_bytes(&self, start: TwoBytes) -> TwoBytes {
		TwoBytes::from((self[start].data as u16) << 8 + self[start.data as usize + 1].data)
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

	pub fn get_f(&self) -> Byte {
		self.data[0xF]
	}
	
	pub fn set_f(&mut self, v: Byte) {
		self.data[0xF] = v;
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

	pub fn set_y(&mut self, v: TwoBytes, value: Byte) {
		self.data[v.shift_right(4).mask_bits(&TwoBytes::from(0xF)).as_usize()] = value;
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
	pub memory: Memory,
	data_registers: DataRegisters,
	adress_register: TwoBytes,
	stack: LinkedList<TwoBytes>,
	pc: TwoBytes,
	pub screen: super::screen::Screen,
	delay_timer: Byte,
	sound_timer: Byte
}
impl State {
	pub fn new_chip8() -> Self {
		let mut new = Self {
			memory: Memory::new(4096),
			data_registers: DataRegisters::new(16),
			adress_register: TwoBytes::new(),
			stack: LinkedList::new(),
			pc: TwoBytes::from(0x200),
			screen: super::screen::Screen::new(64, 32),
			delay_timer: Byte::new(),
			sound_timer: Byte::new()
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

	pub fn interpret_next(&mut self) -> bool {
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
				0x00E0 => {
					self.screen.clear();
					self.pc.increase(2);
				}, 0x00EE => self.pc = self.stack.pop_back().unwrap(),
				_ => {
					todo!("not necessary?");
				}
			}, 0x1 => {
				if self.pc == l3_const {
					return true;
				}
				self.pc = l3_const;
			}, 0x2 => {
				self.stack.push_back(self.pc);
				self.pc = l3_const;
			}, 0xB => {
				self.pc = l3_const;
				self.pc.increase_with_byte(self.data_registers.get_0());
			}, 0x3000 => if x == l2_const {
				self.pc.increase(2)
			}, 0x4000 => if x != l2_const {
				self.pc.increase(2);
			}, 0x5000 => if x == y {
				self.pc.increase(2);
			}, 0x6000 => self.data_registers.set_x(current, l2_const),
			0x7000 => self.data_registers.set_x(current, x.add_byte(y)),
			0x8000 => match l1_const.data {
				0x00 => self.data_registers.set_x(current, y),
				0x01 => self.data_registers.set_x(current, Byte::from(x.data | y.data)),
				0x02 => self.data_registers.set_x(current, Byte::from(x.data & y.data)),
				0x03 => self.data_registers.set_x(current, Byte::from(x.data ^ y.data)),
				0x04 => {
					self.data_registers.set_f(Byte::new());
					if x.data > (0xFF - y.data) {
						self.data_registers.set_f(Byte::from(1));
					}
					self.data_registers.set_x(current, x.add_byte(y));
				}, 0x05 => {
					self.data_registers.set_f(Byte::new());
					if x.data < y.data {
						self.data_registers.set_f(Byte::from(1));
						self.data_registers.set_x(current, Byte::from((x.data as u16 + 0xFF - y.data as u16) as u8));
					} else {
						self.data_registers.set_x(current, Byte::from(x.data - y.data));
					}
				}, 0x06 => {
					self.data_registers.set_f(Byte::from(((current.data & 0b1000_0000) >> 7) as u8));
					self.data_registers.set_x(current, Byte::from(((current.data & 0b0111_1111) << 1) as u8));
				}, 0x07 => {
					self.data_registers.set_f(Byte::new());
					if y.data < x.data {
						self.data_registers.set_f(Byte::from(1));
						self.data_registers.set_x(current, Byte::from((y.data as u16 + 0xFF - x.data as u16) as u8));
					} else {
						self.data_registers.set_x(current, Byte::from(y.data - x.data));
					}
				}, 0x08 => {
					self.data_registers.set_f(Byte::from((current.data & 0b0000_0001) as u8));
					self.data_registers.set_x(current, Byte::from(((current.data & 0b1111_1110) >> 1) as u8));
				},
				_ => panic!("Unknown Variable Operation {}.", current)
			}, 0x9000 => if x.data != y.data {
				self.pc.increase(2);
			}, 0xA000 => self.adress_register = l3_const,
			0xB000 => self.pc = l3_const.add_byte(self.data_registers.get_0()),
			0xC000 => {
				unimplemented!("RAND");
				/*let rand = start.elapsed().unwrap().as_nanos() % (current_opcode & 0x00FF) as u128;
				v[((current_opcode & 0x0F00) >> 8) as usize] = rand as u8;
				pc += 2;*/
			}, 0xD000 => {
				self.data_registers.set_f(Byte::from(0));
				for h in 0..current.mask_bits(&TwoBytes::from(0x000F)).data {
					for i in 0..8 {
						if self.memory[self.adress_register.add(h)].mask_bits(&Byte::from(1).shift_left(7 - i)).shift_right(7 - i).data == 1 {
							let screen_x = x.as_usize() + i as usize;
							let screen_y = y.as_usize() + h as usize;
							if self.screen.get(screen_x, screen_y) {
								self.data_registers.set_f(Byte::from(1));
							}
							self.screen.swap(screen_x, screen_y);
						}
					}
				}
			}, 0xE000 => {
				unimplemented!("KEYS");
				/*if stored_key == match (current_opcode & 0x00FF) as u8 {
					0x9E => true,
					0xA1 => false,
					_ => panic!("Unknown key opcode {:x}.", current_opcode)
				} {
					pc += 2;
				}
				pc += 2;*/
			}, 0xF000 => {
				match l2_const.data as u8 {
					0x07 => self.data_registers.set_x(current, self.delay_timer),
					0x0A => {
						unimplemented!("KEYS");
					}, 0x15 => self.delay_timer = x,
					0x18 => self.sound_timer = x,
					0x1E => self.adress_register.increase_with_byte(x),
					0x29 => self.adress_register = TwoBytes::from(x.data as u16 * 5),
					0x33 => {
						self.memory[self.adress_register.as_usize()] = x.shift_right(5);
						self.memory[self.adress_register.as_usize() + 1] = x.shift_left(3).shift_right(6);
						self.memory[self.adress_register.as_usize() + 2] = x.shift_left(5).shift_right(5);
					}, 0x55 => for i in 0..=x.as_usize() {
						self.memory[self.adress_register.as_usize() + i] = self.data_registers[i];
					}, 0x65 => for i in 0..=x.as_usize() {
						self.data_registers[i] = self.memory[self.adress_register.as_usize() + i];
					}, _ => panic!("Unknown memory opcode {}.", current.data)
				}
			}
		}
		false
	}
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