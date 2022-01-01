use std::{ thread, time, fs };
use std::time::SystemTime;

static FONT: [ u8; 80 ] = [
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
];

fn main() {
	let start = SystemTime::now();

	let mut memory = [ 0u8; 4096 ];
	let mut stack = [ 0u16; 15 ];
	let mut v = [ 0u8; 16 ];
	let mut screen = [ [ false; 64 ]; 32 ];

	FONT.iter().enumerate().for_each(| (i, &b) | memory[i] = b);
	let programm = fs::read("./Pong.ch8").expect("Failed to read programm.");
	programm.iter().enumerate().for_each(| (i, &b) | memory[i + 0x200] = b);
	
	let mut delay_timer = 0;
	let mut sound_timer = 0;

	let mut pc = 0x200u16;
	let mut index_pointer = 0;
	let mut stack_pointer = 0;
	let mut stored_key = false;
	let mut awaiting_key = None;

	while pc < 0x200 + programm.len() as u16 {
		if let Some(register) = awaiting_key {
			v[register] = 0;
			stored_key = true;
			awaiting_key = None;
			continue;
		}

		let start_of_frame = SystemTime::now();
		let current_opcode = (memory[pc as usize] as u16) << 8 | memory[pc as usize + 1] as u16;
		//println!("0x{:x}", current_opcode);
		match current_opcode & 0xF000 {
			0x0000 => {
				match current_opcode & 0x0FFF {
					0x00E0 => {
						//pc += 2;
						todo!("Clear screen");
					},
					0x00EE => {
						pc = stack[stack_pointer - 1];
						stack_pointer -= 1;
					},
					_ => {
						//pc += 2;
						todo!("not necessary?");
					}
				}
			}, 0x1000 => pc = current_opcode & 0x0FFF,
			0x2000 => {
				stack[stack_pointer] = pc;
				stack_pointer += 1;
				pc = current_opcode & 0x0FFF;
			}, 0x3000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				if v[x] == (current_opcode & 0x00FF) as u8 {
					pc += 2;
				}
				pc += 2;
			}, 0x4000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				if v[x] != (current_opcode & 0x00FF) as u8 {
					pc += 2;
				}
				pc += 2;
			}, 0x5000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				let y = ((current_opcode & 0x00F0) >> 4) as usize;
				if v[x] == v[y] {
					pc += 2;
				}
				pc += 2;
			}, 0x6000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				v[x] = (current_opcode & 0x00FF) as u8;
				pc += 2;
			}, 0x7000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				v[x] += (current_opcode & 0x00FF) as u8;
				pc += 2;
			}, 0x8000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				let y = ((current_opcode & 0x00F0) >> 4) as usize;
				match (current_opcode & 0x000F) as u8 {
					0x00 => v[x] = v[y],
					0x01 => v[x] |= v[y],
					0x02 => v[x] &= v[y],
					0x03 => v[x] ^= v[y],
					0x04 => if v[x] > (0xFF - v[y]) {
							v[0xF] = 1;
							v[x] -= v[y];
						} else {
							v[0xF] = 0;
							v[x] += v[y];
						},
					0x05 => if v[x] < v[y] {
							v[0xF] = 1;
							v[x] = v[y] - v[x];
						} else {
							v[0xF] = 0;
							v[x] -= v[y];
						},
					0x07 => if v[x] > v[y] {
							v[0xF] = 1;
							v[x] = v[x] - v[y];
						} else {
							v[0xF] = 0;
							v[x] = v[y] - v[x];
						},
					_ => panic!("Unknown Operation {}.", current_opcode)
				};
				pc += 2;
			}, 0x9000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				let y = ((current_opcode & 0x00F0) >> 4) as usize;
				if v[x] != v[y] {
					pc += 2;
				}
				pc += 2;
			}, 0xA000 => {
				index_pointer = current_opcode & 0x0FFF;
				pc += 2;
			}, 0xB000 => {
				pc = current_opcode & 0x0FFF + v[0] as u16;
			}, 0xC000 => {
				let rand = start.elapsed().unwrap().as_nanos() % (current_opcode & 0x00FF) as u128;
				v[((current_opcode & 0x0F00) >> 8) as usize] = rand as u8;
				pc += 2;
			}, 0xD000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				let y = ((current_opcode & 0x00F0) >> 4) as usize;
				v[0xF] = 0x00;
				for h in 0..=current_opcode & 0x000F {
					for i in 0..8 {
						// get the byte in the memory, 'remove' non-important bits, check if that bit is '1'
						let new_bit = memory[index_pointer as usize + h as usize] & (1 << (x % 8 + i) % 8) == 1 << (x % 8 + i) % 8;
						if screen[y][x + i] && new_bit {
							v[0xF] = 0x01;
						}
						screen[y][x + i] = !new_bit;
					}
				}
				pc += 2;
			}, 0xE000 => {
				if stored_key == match (current_opcode & 0x00FF) as u8 {
					0x9E => true,
					0xA1 => false,
					_ => panic!("Unknown key opcode {}.", current_opcode)
				} {
					pc += 2;
				}
				pc += 2;
			}, 0xF000 => {
				let x = ((current_opcode & 0x0F00) >> 8) as usize;
				match (current_opcode & 0x00FF) as u8 {
					0x07 => {
						v[x] = delay_timer;
					}, 0x0A => {
						awaiting_key = Some(x);
					}, 0x15 => {
						delay_timer = v[x];
					}, 0x18 => {
						sound_timer = v[x];
					}, 0x1E => {
						index_pointer += v[x] as u16;
					}, 0x29 => {
						index_pointer = v[x] as u16 * 5;
					}, 0x33 => {
						memory[index_pointer as usize] = (v[x] & 0b11100000) >> 5;
						memory[index_pointer as usize + 1] = (v[x] & 0b00011000) >> 3;
						memory[index_pointer as usize + 2] = v[x] & 0b00000111;
					}, 0x55 => {
						for i in 0..=x {
							memory[index_pointer as usize + i] = v[i];
						}
					}, 0x65 => {
						for i in 0..=x {
							v[i] = memory[index_pointer as usize + i];
						}
					}, _ => panic!("Unknown memory opcode {}.", current_opcode)
				}
			}
			_ => panic!("Unknown opcode {:x}.", current_opcode)
		}
		
		if delay_timer > 0 {
			delay_timer -= 1;
		}
		if sound_timer > 0 {
			if sound_timer == 1 {
				println!("BEEP!");
			}
			sound_timer -= 1;
		}
		thread::sleep(time::Duration::from_millis((60 - start_of_frame.elapsed().unwrap().as_millis()) as u64));
	}
}