use std::collections::{HashSet, LinkedList};
use std::sync::{Arc, Mutex};
use crate::fixed_bit_numbers::{FixedBitNumber, IntoEmpty};
use crate::logger::LogInfo;
use crate::output::Output;

type Byte = FixedBitNumber<8>;
type Address = FixedBitNumber<16>;

pub trait Interpreter {
    fn new(output: Arc<Mutex<Output>>) -> Self;
    fn next_frame(&mut self);
    fn shutdown(&mut self);
    fn load_memory(&mut self, bytes: Vec<u8>, starting_address: u16);
    fn interpret_next(&mut self);
}

pub struct Chip8Interpreter {
    memory: Vec<Byte>,
    data_registers: Vec<Byte>,
    address_register: Address,
    stack: LinkedList<Address>,
    pc: Address,
    output: Arc<Mutex<Output>>,
    delay_timer: Byte,
    sound_timer: Byte,
    random_numbers: LinkedList<Byte>,
    awaiting_key: Option<usize>,
    finished: bool
}
impl Chip8Interpreter {
    fn get_next_random(&mut self) -> Byte {
        let r = self.random_numbers.pop_front().unwrap();
        self.random_numbers.push_back(r);
        r
    }
}
impl Interpreter for Chip8Interpreter {
    fn new(output: Arc<Mutex<Output>>) -> Self {
        Self {
            memory: [ Byte::new(); 4096 ].to_vec(),
            data_registers: [ Byte::new(); 16 ].to_vec(),
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
            }, awaiting_key: None
        }
    }

    fn next_frame(&mut self) {
        if self.delay_timer.into_u32() > 0 {
            self.delay_timer.decrease_by_u32(1);
        }
        if self.sound_timer.into_u32() > 0 {
            let output = self.output.lock().unwrap();
            self.sound_timer.decrease_by_u32(1);
            if self.sound_timer.into_u32() == 0 {
                output.stop_buzz();
            } else {
                output.buzz();
            }
        }
    }

    fn shutdown(&mut self) {}

    fn load_memory(&mut self, bytes: Vec<u8>, starting_address: u16) {
        for (i, b) in bytes.into_iter().enumerate() {
            self.memory[i + starting_address as usize] = Byte::from(b as u32);
        }
    }

    fn interpret_next(&mut self) {
        if self.finished {
            return;
        }
        if let Some(x) = self.awaiting_key {
            if let Some(c) = self.output.lock().unwrap().get_current_input() {
                self.data_registers[x].set_by_u32(c.into());
                self.awaiting_key = None;
            }
            return;
        }
        let current = Address::from_combined(&self.memory[&self.pc], &self.memory[&self.pc.add_by_u32(1)]);
        let x = &current.get_bitrange(8, 4);
        let vx = self.data_registers[x];
        let vy = self.data_registers[&current.get_bitrange(4, 4)];
        let l3_const = current.get_bitrange(0, 12);
        let l2_const = current.get_bitrange(0, 8);
        let l1_const = current.get_bitrange(0, 4);
        match current.get_bitrange(12, 4).into_u32() {
            0x0 | 0x1 | 0x2 | 0xB => {},
            _ => self.pc.increase_by_u32(2).into_empty()
        };
        match current.get_bitrange(12, 4).into_u32() {
            0x0 => match l3_const.into_u32() {
                0x0E0 => {
                    self.output.lock().unwrap().clear();
                    self.pc.increase_by_u32(2);
                }, 0x0EE => self.pc = self.stack.pop_back().unwrap(),
                _ => panic!("No rom interaction possible (opcode: {}, pc: {})", current, self.pc)
            }, 0x1 => {
                if l3_const != self.pc {
                    self.pc.set(&l3_const);
                } else {
                    format!("Looping around {} - program finished!", self.pc).as_str().log();
                    self.finished = true;
                }
            },
            0x2 => {
                self.stack.push_back(self.pc.add_by_u32(2));
                self.pc = l3_const;
            }, 0x3 => vx.execute_if_equals(&l2_const, || self.pc.increase_by_u32(2).into_empty()),
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
                }, 0x5 => {
                    let f = self.data_registers[x].decrease(&vy);
                    self.data_registers[0xF].set_bool(f);
                }, 0x6 => {
                    let f = self.data_registers[x].shift_right();
                    self.data_registers[0xF].set_bool(f);
                }, 0x7 => {
                    let f = self.data_registers[x].reversed_decrease(&vy);
                    self.data_registers[0xF].set_bool(f);
                }, 0xE => {
                    let f = self.data_registers[x].shift_left();
                    self.data_registers[0xF].set_bool(f);
                }, _ => panic!("Unknown variable opcode {}.", current)
            }, 0x9 => vx.execute_if_not_equals(&vy, || self.pc.increase_by_u32(2).into_empty()),
            0xA => self.address_register.set(&l3_const),
            0xB => self.pc.set(&l3_const.add(&self.data_registers[0])),
            0xC => {
                let nr = &self.get_next_random();
                self.data_registers[x].set(nr);
                self.data_registers[x].and(&l2_const);
            }, 0xD => {
                self.data_registers[0xF].set_bool(false);
                for row in 0..l1_const.into_u32() {
                    for bit in 0..8 {
                        if self.memory[&self.address_register.add_by_u32(row)].get_bit(7 - bit) {
                            let collision = self.output.lock().unwrap().swap(vx.add_by_u8(bit).into_u32(), vy.add_by_u32(row).into_u32());
                            self.data_registers[0xF].set_bool(collision);
                        }
                    }
                }
            }, 0xE => {
                match l2_const.into_u32() {
                    0x9E => if self.output.lock().unwrap().is_pressed(vx.into_u8()) {
                        self.pc.increase_by_u32(2);
                    }, 0xA1 => if !self.output.lock().unwrap().is_pressed(vx.into_u8()) {
                        self.pc.increase_by_u32(2);
                    }, _ => panic!("Unknown opcode {}.", current)
                }
            }, 0xF => {
                let second_hex = current.get_bitrange(8, 4);
                match l2_const.into_u32() {
                    0x07 => self.data_registers[x].set(&self.delay_timer),
                    0x0A => {
                        "Awaiting a key (hex) input ...".log();
                        self.awaiting_key = Some(x.into_usize());
                    }, 0x15 => self.delay_timer.set(&self.data_registers[x]),
                    0x18 => {
                        if vx.into_u32() == 0 {
                            self.output.lock().unwrap().stop_buzz();
                        }
                        self.sound_timer.set(&self.data_registers[x]);
                    },
                    0x1E => self.address_register.increase(&self.data_registers[x]).into_empty(),
                    0x29 => self.address_register.set_by_u32(self.data_registers[x].into_u32() * 5),
                    0x33 => {
                        self.memory[&self.address_register].set_by_u32(vx.into_u32() / 100);
                        self.memory[&self.address_register.add_by_u32(1)].set_by_u32(vx.into_u32() % 100 / 10);
                        self.memory[&self.address_register.add_by_u32(2)].set_by_u32(vx.into_u32() % 10);
                    }, 0x55 => for i in 0..=second_hex.into_usize() {
                        self.memory[&self.address_register.add_by_usize(i)].set(&self.data_registers[i]);
                    }, 0x65 => for i in 0..=second_hex.into_usize() {
                        self.data_registers[i].set(&self.memory[&self.address_register.add_by_usize(i)]);
                    }, _ => panic!("Unknown memory opcode {}.", current)
                }
            }, _ => panic!("Unknown opcode {} at {}.", current, self.pc)
        }
    }
}