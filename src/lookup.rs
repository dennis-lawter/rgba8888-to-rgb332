use std::collections::HashMap;

pub struct LookupTable {
    red_table: HashMap<u8, u8>,
    green_table: HashMap<u8, u8>,
    blue_table: HashMap<u8, u8>,
}

impl LookupTable {
    pub fn new() -> Self {
        let mut red_table: HashMap<u8, u8> = HashMap::new();
        red_table.insert(0x00, 0b000_000_00);
        red_table.insert(0x24, 0b001_000_00);
        red_table.insert(0x49, 0b010_000_00);
        red_table.insert(0x6d, 0b011_000_00);
        red_table.insert(0x92, 0b100_000_00);
        red_table.insert(0xb6, 0b101_000_00);
        red_table.insert(0xdb, 0b110_000_00);
        red_table.insert(0xff, 0b111_000_00);

        let mut green_table: HashMap<u8, u8> = HashMap::new();
        green_table.insert(0x00, 0b000_000_00);
        green_table.insert(0x24, 0b000_001_00);
        green_table.insert(0x49, 0b000_010_00);
        green_table.insert(0x6d, 0b000_011_00);
        green_table.insert(0x92, 0b000_100_00);
        green_table.insert(0xb6, 0b000_101_00);
        green_table.insert(0xdb, 0b000_110_00);
        green_table.insert(0xff, 0b000_111_00);

        let mut blue_table: HashMap<u8, u8> = HashMap::new();
        blue_table.insert(0x00, 0b000_000_00);
        blue_table.insert(0x55, 0b000_000_01);
        blue_table.insert(0xaa, 0b000_000_10);
        blue_table.insert(0xff, 0b000_000_11);

        Self {
            red_table,
            green_table,
            blue_table,
        }
    }

    pub fn get(&self, key: &[u8; 4]) -> Result<Option<u8>, ()> {
        if key[3] == 0x00 {
            // pixel is fully transparent
            return Ok(None);
        } else if key[3] != 0xff {
            // partial transparency is not supported
            return Err(());
        }
        let red_bits = match self.red_table.get(&key[0]) {
            Some(bits) => bits,
            None => return Err(()),
        };
        let green_bits = match self.green_table.get(&key[1]) {
            Some(bits) => bits,
            None => return Err(()),
        };
        let blue_bits = match self.blue_table.get(&key[2]) {
            Some(bits) => bits,
            None => return Err(()),
        };
        Ok(Some(red_bits ^ green_bits ^ blue_bits))
    }
}
