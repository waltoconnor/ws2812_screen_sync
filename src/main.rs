extern crate libusb;

mod get_color;
mod config;
mod monitor;

use std::io::Write;
use std::thread;

use serde::Deserialize;
use serde::Serialize;

use config::load_config;
use get_color::{color_thread, Colors};
pub use get_color::{RGB, ScreenRegion, HealthBar};

use serialport::SerialPort;
use serialport::SerialPortInfo;
use serialport::TTYPort;
use serialport::available_ports;
use serialport::SerialPortType;

fn send_colors(colors: &Colors, dev: &mut TTYPort){
    //println!("{:?}", colors.color_vec);
    let mut buf = [0u8; 360];
    for (idx, c) in colors.color_vec.iter().enumerate() {
        let idx_1 = ((idx & 0xFF00) >> 8) as u8;
        let idx_2 = (idx & 0xFF) as u8;
        let dtype = 1u8;
        let r = c.r;
        let g = c.g;
        let b = c.b;
        //let buf = [dtype, idx_1, idx_2, r, g, b];
        //dev.write(&buf);
        let base_idx = idx * 6;
        buf[base_idx] = dtype;
        buf[base_idx + 1] = idx_1;
        buf[base_idx + 2] = idx_2;
        buf[base_idx + 3] = r;
        buf[base_idx + 4] = g;
        buf[base_idx + 5] = b;
    }

    dev.write(&buf);
    
}

fn main() {
//     println!("Hello, world!");

//     let mut context = libusb::Context::new().unwrap();

//     for mut device in context.devices().unwrap().iter() {
//         let device_desc = device.device_descriptor().unwrap();

//         println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
//             device.bus_number(),
//             device.address(),
//             device_desc.vendor_id(),
//             device_desc.product_id());
//     }

    let cfg = load_config();

    let av_ports = available_ports().expect("NO SERIAL DEVICES ATTACHED");
    println!("{:#?}", av_ports);

    let mut port: Option<&SerialPortInfo> = None;

    while port.is_none() {
        for p in av_ports.iter() {
            match &p.port_type {
                SerialPortType::UsbPort(usb) => {
                    if usb.vid == cfg.target_usb_vid as u16 && usb.pid == cfg.target_usb_pid as u16 {
                        port = Some(p);
                        println!("selected device: {}:{}", usb.vid, usb.pid);
                    }
                }
                _ => { continue; }
            }
        }

        if port.is_none() {
            println!("DID NOT FIND TARGET DEVICE, LOOPING");
        }
        else {
            
            break;
        }

        thread::sleep(std::time::Duration::from_millis(5000));
    }

    println!("opening port: {}", &port.unwrap().port_name);
    let mut port_dev = serialport::new(&port.unwrap().port_name, 115200)
        .timeout(std::time::Duration::from_millis(200))
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .open_native()
        .expect("Failed to open serial port, do you need admin rights?");

    

    let (tx_config, rx_config) = std::sync::mpsc::channel();
    let (tx_colors, rx_colors) = std::sync::mpsc::channel();

    let mut cur_colors: Colors = Colors { color_vec: Vec::new() };

    for i in 0..cfg.num_leds {
        cur_colors.color_vec.push(RGB{r: 0, b: 0, g: 0});
    }
    
    let num_leds = cfg.num_leds;
    let screen = cfg.screen_id;
    let interval_ms = cfg.update_interval_ms;
    thread::spawn(move ||{ 
        color_thread(screen, num_leds, interval_ms, rx_config, tx_colors);
    });
    
    let dst_addr = format!("{}:{}", cfg.net_cfg.dest_ip, cfg.net_cfg.dest_port);

    let mut mon_dev = monitor::find_monitor();

    //monitor::set_mon_mode(&mut mon_dev.as_mut().unwrap());
    monitor::set_mon_test(&mut mon_dev.as_mut().unwrap());

    loop {

        match rx_colors.try_recv() {
            Ok(c) => {
                cur_colors = c;
            }
            _ => {}
        }

        send_colors(&cur_colors, &mut port_dev);

        if mon_dev.is_some() {
            println!("SENDING MON");
            monitor::send_colors(&cur_colors, &mut mon_dev.as_mut().unwrap())
        }

        //println!("CUR COLORS:");
        //println!("{:?}", cur_colors.color_vec);

        // let mut buffer = String::from("l");

        // let cv = &cur_colors.color_vec;

        // buffer += format!("{},{},{}", cv[0].r, cv[0].g, cv[0].b).as_str();
        // for i in 1..cv.len(){
        //     buffer += format!(";{},{},{}", cv[i].r, cv[i].g, cv[i].b).as_str();
        // }

        // buffer += "/n";

        // println!("buffer: {}", buffer);
        // port_dev.write(buffer.as_bytes());

        thread::sleep(std::time::Duration::from_millis(500));

    }


}
