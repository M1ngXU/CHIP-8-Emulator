use std::{ thread, fs };
use std::time::{ SystemTime, Duration };

mod structs;
mod screen;

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
	0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn main() {
	let mut virtual_machine_state = structs::State::new_chip8();

	virtual_machine_state.load_memory(FONT.to_vec(), 0);
	virtual_machine_state.load_memory(fs::read("./roms/spinvaders.ch8").expect("Failed to read programm."), 0x200);
	
	let mut last_frame = SystemTime::now();
	let mut last_opcode = SystemTime::now();
	let fps = 15;
	let opcodes_per_frame = 48;
	let opcodes_execution_time = (1000.0 / fps as f64 / opcodes_per_frame as f64) as u128;

	let mut i = 0;
	while !virtual_machine_state.interpret_next() {
		if i % opcodes_per_frame == 0 {
			virtual_machine_state.next_frame();
		}
		i += 1;
		while last_opcode.elapsed().unwrap().as_millis() < opcodes_execution_time {}
		//thread::sleep(Duration::from_millis(((1000.0 / fps as f64) / opcodes_per_frame as f64 - last_opcode.elapsed().unwrap().as_millis() as f64).max(0.0) as u64));
		last_opcode = SystemTime::now();
	}
	virtual_machine_state.shut_down();
}