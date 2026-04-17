pub const TEXT_GRID_WIDTH: usize = 80;
pub const TEXT_GRID_HEIGHT: usize = 25;
pub const TEXT_GRID_SIZE: usize = TEXT_GRID_WIDTH * TEXT_GRID_HEIGHT;
// Each cell in the text grid is 2 bytes: one for the ASCII character and one for the color
// attribute.
pub const TEXT_GRID_BYTES: usize = TEXT_GRID_SIZE * 2 - 1;
pub const TEXT_GRID_START: u32 = 0x001F0000;
pub const TEXT_GRID_END: u32 = TEXT_GRID_START + TEXT_GRID_BYTES as u32;
pub const UART_ADDR: u32 = 0x001F8000;
pub const INPUT_BYTE_FIRST: u32 = 0x001F8001;
pub const INPUT_BYTE_SECOND: u32 = 0x001F8002;

pub struct Memory {
    data: Vec<u8>,
    pub using_macroquad: bool,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            data: vec![0; 1024 * 1024 * 2],
            using_macroquad: true,
        }
    }

    pub fn read_u8(&self, addr: u32) -> u8 {
        match addr {
            UART_ADDR => 0,
            INPUT_BYTE_FIRST => {
                if self.using_macroquad {
                    let key = macroquad::input::get_keys_pressed().into_iter().next();
                    if let Some(key) = key {
                        (key as u16).to_le_bytes()[0]
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            INPUT_BYTE_SECOND => {
                if self.using_macroquad {
                    let key = macroquad::input::get_keys_pressed().into_iter().next();
                    if let Some(key) = key {
                        (key as u16).to_le_bytes()[1]
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            _ => self.data[addr as usize],
        }
    }

    pub fn read_u32(&self, addr: u32) -> u32 {
        let byte_1 = self.read_u8(addr) as u32;
        let byte_2 = self.read_u8(addr + 1) as u32;
        let byte_3 = self.read_u8(addr + 2) as u32;
        let byte_4 = self.read_u8(addr + 3) as u32;
        u32::from_le_bytes([byte_1 as u8, byte_2 as u8, byte_3 as u8, byte_4 as u8])
    }

    pub fn read_u64(&self, addr: u32) -> u64 {
        let byte_1 = self.read_u8(addr) as u64;
        let byte_2 = self.read_u8(addr + 1) as u64;
        let byte_3 = self.read_u8(addr + 2) as u64;
        let byte_4 = self.read_u8(addr + 3) as u64;
        let byte_5 = self.read_u8(addr + 4) as u64;
        let byte_6 = self.read_u8(addr + 5) as u64;
        let byte_7 = self.read_u8(addr + 6) as u64;
        let byte_8 = self.read_u8(addr + 7) as u64;
        u64::from_le_bytes([
            byte_1 as u8,
            byte_2 as u8,
            byte_3 as u8,
            byte_4 as u8,
            byte_5 as u8,
            byte_6 as u8,
            byte_7 as u8,
            byte_8 as u8,
        ])
    }

    pub fn write_u8(&mut self, addr: u32, value: u8) {
        match addr {
            TEXT_GRID_START..=TEXT_GRID_END => {
                self.data[addr as usize] = value;
            }
            UART_ADDR => {
                print!("{}", value as char);
            }
            _ => {
                self.data[addr as usize] = value;
            }
        }
    }

    pub fn write_u32(&mut self, addr: u32, value: u32) {
        let bytes = value.to_le_bytes();
        self.write_u8(addr, bytes[0]);
        self.write_u8(addr + 1, bytes[1]);
        self.write_u8(addr + 2, bytes[2]);
        self.write_u8(addr + 3, bytes[3]);
    }

    pub fn write_u64(&mut self, addr: u32, value: u64) {
        let bytes = value.to_le_bytes();
        self.write_u8(addr, bytes[0]);
        self.write_u8(addr + 1, bytes[1]);
        self.write_u8(addr + 2, bytes[2]);
        self.write_u8(addr + 3, bytes[3]);
        self.write_u8(addr + 4, bytes[4]);
        self.write_u8(addr + 5, bytes[5]);
        self.write_u8(addr + 6, bytes[6]);
        self.write_u8(addr + 7, bytes[7]);
    }

    pub fn load(&mut self, addr: u32, data: &[u8]) {
        let end = (addr as usize) + data.len();
        self.data[addr as usize..end].copy_from_slice(data);
    }

    pub fn text_grid(&self) -> &[u8] {
        &self.data[TEXT_GRID_START as usize..=TEXT_GRID_END as usize]
    }
}
