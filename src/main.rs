use std::fs;
use std::time::SystemTime;
use simple_logger::SimpleLogger;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;

mod structs;
mod output;
mod event_manager;
mod keyboard_state;
mod mouse_state;

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

static FPS: f64 = 60.0;

fn main() {
	SimpleLogger::new().init().unwrap();
	let mut virtual_machine_state = structs::State::new_chip8();
	let event_manager = virtual_machine_state.output.get_event_manager_pointer();
	let mut keyboard_state = virtual_machine_state.output.get_keyboard_state();
	let mut mouse_state = virtual_machine_state.output.get_mouse_state();

	virtual_machine_state.load_memory(FONT.to_vec(), 0);
	virtual_machine_state.load_memory(
		fs::read(std::env::args().nth(1).unwrap_or("./roms/spinvaders.ch8".to_string())
	).expect("Failed to read program."), 0x200);

	let mut last_opcode = SystemTime::now();
	let mut fps = FPS;
	let log_speed = | f: f64 | log::info!("Current speed: {}%", (f / FPS * 100.0).ceil());
	let mut pause = false;
	let opcodes_per_frame = 12;
	let mut cheating = false;

	let mut i = 0;
	while !event_manager.lock().unwrap().is_terminating() {
		mouse_state.update();
		if cheating {
			if let Some(&m) = mouse_state.currently_pressed.iter().next() {
				let (x, y) = mouse_state.coordinates;
				match m {
					MouseButton::Left => {
						virtual_machine_state.output.set(x as u32, y as u32, true);
						continue;
					}, MouseButton::Right => {
						virtual_machine_state.output.set(x as u32, y as u32, false);
						continue;
					},
					_ => {}
				}
			}
		}
		keyboard_state.update();
		if keyboard_state.just_pressed.contains(&Scancode::F1.into()) {
			fps = FPS;
			log_speed(fps);
		}
		if keyboard_state.just_pressed.contains(&Scancode::F2.into()) {
			fps /= 1.1;
			log_speed(fps);
		}
		if keyboard_state.just_pressed.contains(&Scancode::F3.into()) {
			fps *= 1.1;
			log_speed(fps);
		}
		if keyboard_state.just_pressed.contains(&Scancode::F4.into()) {
			cheating = !cheating;
			if cheating {
				log::warn!("CHEAT-MODE turned on!!! Drawing onto the screen modifies collisions ...");
			} else {
				log::info!("CHEAT-MODE turned off!!!");
			}
		}
		if keyboard_state.just_pressed.contains(&Scancode::Escape.into()) {
			pause = !pause;
			if pause {
				log::info!("Paused emulation ...")
			} else {
				log::info!("Unpaused emulation ...")
			}
		}
		if pause || !event_manager.lock().unwrap().is_focused() {
			continue;
		}

		virtual_machine_state.interpret_next();
		if i % opcodes_per_frame == 0 {
			virtual_machine_state.next_frame();
		}
		i += 1;
		while last_opcode.elapsed().unwrap().as_millis() < (1000.0 / fps / opcodes_per_frame as f64) as u128 {}
		last_opcode = SystemTime::now();
	}
	virtual_machine_state.shutdown();
}