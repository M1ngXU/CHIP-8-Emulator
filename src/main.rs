use std::{ thread, fs };
use std::time::{ SystemTime, Duration };

mod structs;
mod screen;
mod event_manager;
mod audio_manager;

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
	let screen = screen::Screen::new(64, 32, 25);
	let event_manager = screen.get_event_manager_pointer();
	let mut virtual_machine_state = structs::State::new_chip8(screen);

	virtual_machine_state.load_memory(FONT.to_vec(), 0);
	virtual_machine_state.load_memory(
		fs::read(std::env::args().nth(1).unwrap_or("./roms/pong.ch8".to_string())
	).expect("Failed to read program."), 0x200);

	let mut last_opcode = SystemTime::now();
	let fps = 60;
	let opcodes_per_frame = 12;
	let opcodes_execution_time = (1000.0 / fps as f64 / opcodes_per_frame as f64) as u128;

	let mut i = 0;
	while !virtual_machine_state.interpret_next() && !event_manager.lock().unwrap().is_terminating() {
		if i % opcodes_per_frame == 0 {
			virtual_machine_state.next_frame();
		}
		i += 1;
		while last_opcode.elapsed().unwrap().as_millis() < opcodes_execution_time {}
		last_opcode = SystemTime::now();
	}
	virtual_machine_state.shut_down();
}