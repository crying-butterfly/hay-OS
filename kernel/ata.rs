use x86_64::instructions::port::Port;

// reads exactly one sector from the hard drive 
pub fn read_sector(lba: u32, buffer: &mut [u8; 512]) {
    unsafe {
        let mut sector_count_port = Port::<u8>::new(0x1F2);
        let mut lba_lo_port = Port::<u8>::new(0x1F3);
        let mut lba_mid_port = Port::<u8>::new(0x1F4);
        let mut lba_hi_port = Port::<u8>::new(0x1F5);
        let mut drive_port = Port::<u8>::new(0x1F6);
        let mut command_port = Port::<u8>::new(0x1F7);
        let mut data_port = Port::<u16>::new(0x1F0);

        drive_port.write(0xE0 | ((lba >> 24) & 0x0F) as u8); // Master Drive
        sector_count_port.write(1); // 1 sector
        lba_lo_port.write(lba as u8);
        lba_mid_port.write((lba >> 8) as u8);
        lba_hi_port.write((lba >> 16) as u8);
        command_port.write(0x20); // Command: Read with Retry

        // wait till the drive is ready
        while command_port.read() & 0x08 == 0 {}

        for i in 0..256 {
            let word = data_port.read();
            buffer[i * 2] = (word & 0xFF) as u8;
            buffer[i * 2 + 1] = (word >> 8) as u8;
        }

    }
}

pub fn write_sector(lba: u32, buffer: &[u8; 512]) {
    unsafe {
        let mut sector_count_port = Port::<u8>::new(0x1F2);
        let mut lba_lo_port = Port::<u8>::new(0x1F3);
        let mut lba_mid_port = Port::<u8>::new(0x1F4);
        let mut lba_hi_port = Port::<u8>::new(0x1F5);
        let mut drive_port = Port::<u8>::new(0x1F6);
        let mut command_port = Port::<u8>::new(0x1F7);
        let mut data_port = Port::<u16>::new(0x1F0);

        drive_port.write(0xE0 | ((lba >> 24) & 0x0F) as u8);
        sector_count_port.write(1);
        lba_lo_port.write(lba as u8);
        lba_mid_port.write((lba >> 16) as u8);
        lba_hi_port.write((lba >> 16) as u8);
        command_port.write(0x30); // Command Write with Retry

        while command_port.read() & 0x08 == 0 {}

        for i in 0..256 {
            let word = (buffer[i * 2] as u16) | ((buffer[i * 2 + 1] as u16) << 8);
            data_port.write(word);
        }
    }
}