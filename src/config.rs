use std::path::Path;
use std::{collections::HashMap, net::Ipv4Addr};
use std::env;
use std::fs;

use crate::{HealthBar, RGB};
use serde::Deserialize;
use serde::Serialize;


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub num_leds: u32, //number of leds in strip being controlled
    pub screen_id: u32, //x 11 screen id
    pub update_interval_ms: u32, //ms between taking screenshots
    pub net_cfg: NetConfig, //where to send results to
    pub games: HashMap<String, HealthBar>, //list of games and their health bar configs
    pub target_usb_vid: u32,
    pub target_usb_pid: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetConfig {
    pub dest_ip: Ipv4Addr,
    pub dest_port: u16
}

fn default_config() -> Config {
    let mut cfg = Config {
        num_leds: 60,
        screen_id: 0,
        update_interval_ms: 500,
        net_cfg: NetConfig { dest_ip: Ipv4Addr::new(10, 0, 0, 128), dest_port: 6854 },
        games: HashMap::new(),
        target_usb_vid: 0x10C4,
        target_usb_pid: 0xEA60
    };

    cfg.games.insert(String::from("League"), HealthBar {
        program_name: String::from("League of Legends.exe"),
        healthy_color: RGB {r: 0x00, g: 0xFF, b: 0x00},
        unhealthy_color: RGB {r: 0x00, g: 0x00, b: 0x00},
        output_healthy: RGB {r: 0x00, g: 0xFF, b: 0x00},
        output_unhealthy: RGB {r: 0xAA, g: 0x00, b:0x00},
        left_x: 100,
        right_x: 500,
        top_y: 1800,
        bottom_y: 1900,
        l2r: true,
        max_health_ratio_healthy: 1.0,
        priority: 1
    });

    cfg
    
}

pub fn load_config() -> Config {
    let base_path = env::var("HOME").expect("$HOME is not set ?????");
    let file_path = base_path + "/.config/kotu-m2-driver.yml";

    let path = Path::new(&file_path);
    if !path.exists() {
        println!("No config found, creating one");
        let cfg = default_config();
        let cfg_string = serde_yaml::to_string(&cfg).expect("Failed to deserialze demo config");
        fs::write(path, cfg_string).expect("Failed to write demo config");
        return cfg;
    }
    else {
        println!("Config found:");
        let config_text = fs::read_to_string(path).expect("Failed to read existing config file");
        let cfg = serde_yaml::from_str(config_text.as_str()).expect("Failed to parse existing config");
        println!("{:#?}", cfg);
        return cfg;
    }

}
