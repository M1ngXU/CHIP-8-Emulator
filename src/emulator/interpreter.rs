use std::collections::{HashSet, LinkedList};

use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;

use crate::emulator::fixed_bit_numbers::{FixedBitNumber, IntoEmpty};
use crate::sdl2_interaction::event_manager::Event;
use crate::sdl2_interaction::output::Output;
use crate::sdl2_interaction::pressed_key::{HexToScancode, ScancodeToHex};
use crate::sdl2_interaction::screen::{Chip8BoolToColor, Chip8ColorToBool};
use crate::LogInfo;

pub type Byte = FixedBitNumber<8>;
pub type Address = FixedBitNumber<16>;

#[derive(Clone, Debug)]
pub enum InterpreterEvent {
    SetPixel(usize, usize, Color),
    RedrawAll,
    QuickSave,
    QuickLoad,
    Save,
    Load,
    Any,
}
impl Event for InterpreterEvent {
    fn is_any(&self) -> bool {
        matches!(&self, &InterpreterEvent::Any)
    }
}

pub trait Interpreter {
    fn new(output: Output) -> Self;
    fn next_frame(&mut self);
    fn shutdown(&mut self);
    fn load_memory(&mut self, bytes: Vec<u8>, starting_address: u16);
    fn interpret_next(&mut self, pressed_keys: &HashSet<Scancode>);
    fn get_output_mut(&mut self) -> &mut Output;
    fn get_output(&self) -> &Output;
}

pub struct Chip8Interpreter {
    memory: Vec<Byte>,
    data_registers: Vec<Byte>,
    address_register: Address,
    stack: LinkedList<Address>,
    pc: Address,
    output: Output,
    delay_timer: Byte,
    sound_timer: Byte,
    random_numbers: LinkedList<Byte>,
    awaiting_key: Option<usize>,
    finished: bool,
}
impl Chip8Interpreter {
    fn get_next_random(&mut self) -> Byte {
        let r = self.random_numbers.pop_front().unwrap();
        self.random_numbers.push_back(r);
        r
    }

    pub fn save(&self) -> Vec<u8> {
        "Saving ...".log();
        let mut save = Vec::new();
        save.append(
            &mut self
                .random_numbers
                .iter()
                .map(|b| b.into_u8())
                .collect::<Vec<u8>>(),
        );
        save.append(&mut self.memory.iter().map(|b| b.into_u8()).collect::<Vec<u8>>());
        let mut screen = self
            .get_output()
            .get_screen()
            .get_pixels()
            .into_iter()
            .flat_map(|(y, r)| {
                r.into_iter()
                    .filter(|&(_, c)| c.into_bool())
                    .fold(Vec::new(), |mut a, (x, _)| {
                        a.push(x as u8);
                        a.push(y as u8);
                        a
                    })
            })
            .collect::<Vec<u8>>();
        save.push(self.get_output().get_screen().get_scale() as u8);
        save.push(self.get_output().get_screen().get_scroll_down() as u8);
        save.push(self.get_output().get_screen().get_scroll_side() as u8);
        save.push(((screen.len() & 0xFF00) >> 8) as u8);
        save.push((screen.len() & 0xFF) as u8);
        save.append(&mut screen);
        save.push(self.address_register.get_bitrange(8, 8).into_u8());
        save.push(self.address_register.get_bitrange(0, 8).into_u8());
        save.append(
            &mut self
                .data_registers
                .clone()
                .iter()
                .map(|b| b.into_u8())
                .collect::<Vec<u8>>(),
        );
        save.push(self.sound_timer.into_u8());
        save.push(self.delay_timer.into_u8());
        save.push(self.pc.get_bitrange(8, 8).into_u8());
        save.push(self.pc.get_bitrange(0, 8).into_u8());
        if let Some(d) = self.awaiting_key {
            save.push(0xF0 + d as u8);
        } else {
            save.push(0x00);
        }
        save.append(
            &mut self
                .stack
                .clone()
                .into_iter()
                .flat_map(|s| {
                    [
                        s.get_bitrange(8, 8).into_u8(),
                        s.get_bitrange(0, 8).into_u8(),
                    ]
                })
                .collect::<Vec<u8>>(),
        );
        save
    }

    pub fn reset(&mut self) {
        self.finished = false;
        self.pc = Address::from(0x200);
        self.output.clear();
    }

    pub fn load(&mut self, data: Vec<u8>) {
        "Loading ...".log();
        self.finished = false;
        self.random_numbers = LinkedList::from_iter(data[..256].iter().map(|b| Byte::from_u8(*b)));
        self.memory = data[256..(4096 + 256)]
            .iter()
            .map(|b| Byte::from_u8(*b))
            .collect::<Vec<Byte>>();
        self.get_output_mut()
            .get_screen_mut()
            .set_scale(data[4096 + 256] as usize);
        self.get_output_mut()
            .get_screen_mut()
            .set_scroll_down(data[4096 + 256 + 1] as usize);
        self.get_output_mut()
            .get_screen_mut()
            .set_scroll_side(data[4096 + 256 + 2] as usize);
        let screen_end =
            (4096 + 256 + 5 + ((data[4096 + 256 + 3] as u16) << 8) + data[4096 + 256 + 4] as u16)
                as usize;
        let screen = &data[(4096 + 256 + 5)..screen_end];
        self.output.get_screen_mut().clear();
        let mut x = None;
        for b in screen {
            if let Some(cur_x) = x {
                x = None;
                self.get_output_mut().get_screen_mut().set(
                    cur_x as usize,
                    *b as usize,
                    true.into_color(),
                );
            } else {
                x = Some(*b);
            }
        }
        self.address_register = Address::from_combined(
            &Byte::from_u8(data[screen_end]),
            &Byte::from_u8(data[screen_end + 1]),
        );
        self.data_registers = data[(screen_end + 2)..(screen_end + 2 + 16)]
            .iter()
            .map(|b| Byte::from_u8(*b))
            .collect::<Vec<Byte>>();
        self.sound_timer = Byte::from_u8(data[(screen_end + 2 + 16)]);
        self.delay_timer = Byte::from_u8(data[(screen_end + 2 + 16 + 1)]);
        self.pc = Address::from_combined(
            &Byte::from_u8(data[(screen_end + 2 + 16 + 2)]),
            &Byte::from_u8(data[(screen_end + 2 + 16 + 3)]),
        );
        if data[(screen_end + 2 + 16 + 3 + 1)] == 0 {
            self.awaiting_key = None;
        } else {
            self.awaiting_key = Some((0xF & data[(screen_end + 2 + 16 + 3 + 1)]) as usize);
        }
        self.stack = LinkedList::new();
        let mut a = Byte::new();
        let mut in_mid_bit = false;
        for b in data[(screen_end + 2 + 16 + 3 + 1)..].iter() {
            if !in_mid_bit {
                a = Byte::from_u8(*b);
                in_mid_bit = true;
            } else {
                self.stack
                    .push_back(Address::from_combined(&a, &Byte::from_u8(*b)));
                a = Byte::new();
                in_mid_bit = false;
            }
        }
    }
}
impl Interpreter for Chip8Interpreter {
    fn new(output: Output) -> Self {
        Self {
            memory: [Byte::new(); 4096].to_vec(),
            data_registers: [Byte::new(); 16].to_vec(),
            address_register: Address::new(),
            stack: LinkedList::new(),
            pc: Address::from(0x200),
            output,
            delay_timer: Byte::new(),
            sound_timer: Byte::new(),
            finished: false,
            random_numbers: {
                let start = std::time::SystemTime::now();
                let mut a = HashSet::new();
                while a.len() < 256 {
                    let e = start.elapsed().unwrap();
                    a.insert(((e.as_nanos() / e.as_millis().max(1)) % 256) as u8);
                }
                LinkedList::from_iter(a.drain().map(Byte::from_u8))
            },
            awaiting_key: None,
        }
    }

    fn next_frame(&mut self) {
        if self.delay_timer.into_u32() > 0 {
            self.delay_timer.decrease_by_u32(1);
        }
        if self.sound_timer.into_u32() > 0 {
            self.sound_timer.decrease_by_u32(1);
            if self.sound_timer.into_u32() == 0 {
                self.output.stop_buzz();
            } else {
                self.output.buzz();
            }
        }
    }

    fn shutdown(&mut self) {}

    fn load_memory(&mut self, bytes: Vec<u8>, starting_address: u16) {
        for (i, b) in bytes.into_iter().enumerate() {
            self.memory[i + starting_address as usize] = Byte::from(b as u32);
        }
    }

    fn interpret_next(&mut self, pressed_keys: &HashSet<Scancode>) {
        if self.finished {
            return;
        }
        if let Some(x) = self.awaiting_key {
            if let Some(c) = pressed_keys.iter().find_map(|k| k.try_into_hex()) {
                self.data_registers[x].set_by_u32(c as u32);
                self.awaiting_key = None;
            }
            return;
        }
        let current = Address::from_combined(&self.memory[&self.pc], &self.memory[&self.pc + 1]);
        let x = &current.get_bitrange(8, 4);
        let vx = self.data_registers[x];
        let vy = self.data_registers[&current.get_bitrange(4, 4)];
        let l3_const = current.get_bitrange(0, 12);
        let l2_const = current.get_bitrange(0, 8);
        let l1_const = current.get_bitrange(0, 4);
        match current.get_bitrange(12, 4).into_u32() {
            0x0 | 0x1 | 0x2 | 0xB => {}
            _ => self.pc.increase_by_u32(2).into_empty(),
        };
        match current.get_bitrange(12, 4).into_u32() {
            0x0 => {
                if l2_const.get_bitrange(4, 4).into_usize() == 0xC {
                    self.output.scroll_down(l1_const.into_usize() as isize);
                    self.pc.increase_by_u32(2);
                } else {
                    match l3_const.into_u32() {
                        0x0 => self.pc.increase_by_u32(2).into_empty(),
                        0x0E0 => {
                            self.output.clear();
                            self.pc.increase_by_u32(2);
                        }
                        0x0EE => self.pc = self.stack.pop_back().unwrap(),
                        0x0FD => self.finished = true,
                        0x0FB => {
                            self.output.scroll_side(4);
                            self.pc.increase_by_u32(2);
                        }
                        0x0FC => {
                            self.output.scroll_side(-4);
                            self.pc.increase_by_u32(2);
                        }
                        0x0FE => {
                            self.output.get_screen_mut().set_scale(2);
                            self.pc.increase_by_u32(2);
                        }
                        0x0FF => {
                            self.output.get_screen_mut().set_scale(1);
                            self.pc.increase_by_u32(2);
                        }
                        _ => panic!(
                            "No rom interaction possible (opcode: {}, pc: {})",
                            current, self.pc
                        ),
                    }
                }
            }
            0x1 => {
                if l3_const != self.pc {
                    self.pc.set(&l3_const);
                } else {
                    format!("Looping around {} - program finished!", self.pc)
                        .as_str()
                        .log();
                    self.finished = true;
                }
            }
            0x2 => {
                self.stack.push_back(&self.pc + 2);
                self.pc = l3_const;
            }
            0x3 => vx.execute_if_equals(&l2_const, || self.pc.increase_by_u32(2).into_empty()),
            0x4 => vx.execute_if_not_equals(&l2_const, || self.pc.increase_by_u32(2).into_empty()),
            0x5 => vx.execute_if_equals(&vy, || self.pc.increase_by_u32(2).into_empty()),
            0x6 => self.data_registers[x].set(&l2_const),
            0x7 => self.data_registers[x].increase(&l2_const).into_empty(),
            0x8 => match l1_const.into_u32() {
                0x0 => self.data_registers[x].set(&vy),
                0x1 => self.data_registers[x].or(&vy),
                0x2 => self.data_registers[x].and(&vy),
                0x3 => self.data_registers[x].xor(&vy),
                0x4 => {
                    let f = self.data_registers[x].increase(&vy);
                    self.data_registers[0xF].set_bool(f);
                }
                0x5 => {
                    let f = self.data_registers[x].decrease(&vy);
                    self.data_registers[0xF].set_bool(f);
                }
                0x6 => {
                    let f = self.data_registers[x].shift_right();
                    self.data_registers[0xF].set_bool(f);
                }
                0x7 => {
                    let f = self.data_registers[x].reversed_decrease(&vy);
                    self.data_registers[0xF].set_bool(f);
                }
                0xE => {
                    let f = self.data_registers[x].shift_left();
                    self.data_registers[0xF].set_bool(f);
                }
                _ => panic!("Unknown variable opcode {}.", current),
            },
            0x9 => vx.execute_if_not_equals(&vy, || self.pc.increase_by_u32(2).into_empty()),
            0xA => self.address_register.set(&l3_const),
            0xB => self
                .pc
                .set_take_ownership(&l3_const + &self.data_registers[0]),
            0xC => {
                let nr = &self.get_next_random();
                self.data_registers[x].set(nr);
                self.data_registers[x].and(&l2_const);
            }
            0xD => {
                self.data_registers[0xF].set_bool(false);
                if l1_const.into_usize() == 0 && self.output.get_screen().get_scale() == 1 {
                    for row in 0..16 {
                        for offset in 0..=1u32 {
                            for pix in 0..8 {
                                if self.memory[&self.address_register + (row + offset)]
                                    .get_bit(7 - pix)
                                {
                                    let collision = self.output.swap(
                                        (&vx + pix as u32).into_usize(),
                                        (&vy + row).into_usize(),
                                    );
                                    self.data_registers[0xF].set_bool(collision);
                                }
                            }
                        }
                    }
                } else {
                    for row in 0..l1_const.into_u32() {
                        for bit in 0..8 {
                            if self.memory[&self.address_register + row].get_bit(7 - bit) {
                                let collision = self.output.swap(
                                    (&vx + bit as u32).into_usize(),
                                    (&vy + row).into_usize(),
                                );
                                self.data_registers[0xF].set_bool(collision);
                            }
                        }
                    }
                }
            }
            0xE => match l2_const.into_u32() {
                0x9E => {
                    if let Some(k) = vx.into_u8().try_into_scancode() {
                        if pressed_keys.contains(&k) {
                            self.pc.increase_by_u32(2);
                        }
                    }
                }
                0xA1 => {
                    if let Some(k) = vx.into_u8().try_into_scancode() {
                        if !pressed_keys.contains(&k) {
                            self.pc.increase_by_u32(2);
                        }
                    } else {
                        self.pc.increase_by_u32(2);
                    }
                }
                _ => panic!("Unknown opcode {}.", current),
            },
            0xF => {
                let second_hex = current.get_bitrange(8, 4);
                match l2_const.into_u32() {
                    0x07 => self.data_registers[x].set(&self.delay_timer),
                    0x0A => {
                        "Awaiting a key (hex) input ...".log();
                        self.awaiting_key = Some(x.into_usize());
                    }
                    0x15 => self.delay_timer.set(&self.data_registers[x]),
                    0x18 => {
                        if vx.into_u32() == 0 {
                            self.output.stop_buzz();
                        }
                        self.sound_timer.set(&self.data_registers[x]);
                    }
                    0x1E => self
                        .address_register
                        .increase(&self.data_registers[x])
                        .into_empty(),
                    0x29 => self
                        .address_register
                        .set_by_u32(self.data_registers[x].into_u32() * 5),
                    0x30 => self
                        .address_register
                        .set_by_u32(self.data_registers[x].into_u32() * 10 + 80),
                    0x33 => {
                        self.memory[&self.address_register].set_by_u32(vx.into_u32() / 100);
                        self.memory[&self.address_register + 1]
                            .set_by_u32(vx.into_u32() % 100 / 10);
                        self.memory[&self.address_register + 2].set_by_u32(vx.into_u32() % 10);
                    }
                    0x55 => {
                        for i in 0..=second_hex.into_u32() {
                            self.memory[&self.address_register + i]
                                .set(&self.data_registers[i as usize]);
                        }
                    }
                    0x65 => {
                        for i in 0..=second_hex.into_u32() {
                            self.data_registers[i as usize]
                                .set(&self.memory[&self.address_register + i]);
                        }
                    }
                    _ => panic!("Unknown memory opcode {}.", current),
                }
            }
            _ => panic!("Unknown opcode {} at {}.", current, self.pc),
        }
    }

    fn get_output_mut(&mut self) -> &mut Output {
        &mut self.output
    }

    fn get_output(&self) -> &Output {
        &self.output
    }
}
