use hidapi::*;

use crate::get_color::Colors;


pub fn find_monitor() -> Option<HidDevice> {
    let VID: u16 = 0x043e;
    let PID: u16 = 0x9a8a;

    let hid = HidApi::new().ok()?;
    println!("hid = {:?}", hid.check_error());
    let mon = hid.open(VID, PID).ok()?;
    println!("hid_dev = {:?}", hid.check_error());

    Some(mon)
}

pub fn set_mon_mode(mon: &mut HidDevice){
    let mut cmd = [0u8; 64];
    cmd[0] = 0x53;
    cmd[1] = 0x43;
    cmd[2] = 0xca;
    cmd[3] = 0x02;
    cmd[4] = 0x02;
    cmd[5] = 0x03;
    cmd[6] = 0x08;
    cmd[7] = 0xd1;
    cmd[8] = 0x45;
    cmd[9] = 0x44;

    
    mon.write(&cmd);
    
}

pub fn set_mon_test(mon: &mut HidDevice){
    let mut cmd = [0u8; 65];
    cmd[1] = 0x53;
    cmd[2] = 0x43;
    cmd[3] = 0xca;
    cmd[4] = 0x02;
    cmd[5] = 0x02;
    cmd[6] = 0x03;
    cmd[7] = 0x01;
    cmd[8] = 0xd8;
    cmd[9] = 0x45;
    cmd[10] = 0x44;

    for i in 0..255u8 {
        cmd[0] = i;
        let val = mon.write(&cmd);
        println!("{} = {:?}",i, val);
    }
    
}

fn gen_monitor_colors(cur_colors: &Colors, out: &mut [u8; 144]){
    let mut out_idx: usize = 0;
    for i in 0..48usize {
        let c = &cur_colors.color_vec[i];
        out[out_idx] = c.r;
        out_idx += 1;
        out[out_idx] = c.g;
        out_idx += 1;
        out[out_idx] = c.b;
        out_idx += 1;
    }
}

fn calc_crc(v: &Vec<u8>) -> u32{
    let mut crc: u32 = 0;
    for byte in v.iter(){
        crc ^= *byte as u32;
        for i in 0..8 {
            crc <<= 1;
            if crc & 0x100 > 0{
                crc ^= 0x101;
            }
        }
    }
    return crc;
}

pub fn send_colors(cur_colors: &Colors, mon: &mut HidDevice){
    let cmd_base: [u8; 6] = [0x53, 0x43, 0xc1, 0x02, 0x91, 0x00];
    let mut cmd_colors: [u8; 144] = [0; 144];
    gen_monitor_colors(cur_colors, &mut cmd_colors);
    let mut v = Vec::new();
    v.extend_from_slice(&cmd_base);
    v.extend_from_slice(&cmd_colors);
    let crc = calc_crc(&v);
    let crc_bytes = crc.to_be_bytes();
    v.extend_from_slice(&crc_bytes);
    let cmd_tail: [u8; 2] = [0x45, 0x44];
    v.extend_from_slice(&cmd_tail);
    
    let mut cmd1 = [0u8; 64];
    let mut cmd2 = [0u8; 64];
    let mut cmd3 = [0u8; 64];

    for i in 0..64usize {
        cmd1[i] = v[i];
    }

    for i in 0..64usize {
        cmd2[i] = v[i + 64];
    }

    for i in 0usize..(64 - 39) {
        cmd3[i] = v[i + 128];
    }

    mon.write(&cmd1);
    mon.write(&cmd2);
    mon.write(&cmd3);
}