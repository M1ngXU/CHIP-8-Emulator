use sdl2::keyboard::Scancode;

static KEYBOARD_LAYOUT: [ Scancode; 16 ] = [
    Scancode::Kp1, Scancode::Kp2, Scancode::Kp3, Scancode::Kp4,
    Scancode::Q, Scancode::W, Scancode::E, Scancode::R,
    Scancode::A, Scancode::S, Scancode::D, Scancode::F,
    Scancode::Z, Scancode::X, Scancode::C, Scancode::V
];
static HEX_LAYOUT: [ u8; 16 ] = [
    0x1, 0x2, 0x3, 0xC,
    0x4, 0x5, 0x6, 0xD,
    0x7, 0x8, 0x9, 0xE,
    0xA, 0x0, 0xB, 0xF
];
pub trait ScancodeToHex {
    fn try_into_hex(&self) -> Option<u8>;
}
impl ScancodeToHex for Scancode {
    fn try_into_hex(&self) -> Option<u8> {
        KEYBOARD_LAYOUT.iter()
            .position(| s | s == self)
            .map(| i | HEX_LAYOUT[i])
    }
}
pub trait HexToScancode {
    fn try_into_scancode(&self) -> Option<Scancode>;
}
impl HexToScancode for u8 {
    fn try_into_scancode(&self) -> Option<Scancode> {
        HEX_LAYOUT.iter()
            .position(| n | n == self)
            .map(| i | KEYBOARD_LAYOUT[i])
    }
}