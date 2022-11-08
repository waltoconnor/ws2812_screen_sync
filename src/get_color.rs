use std::{sync::mpsc::{Receiver, Sender}, thread, time::Instant};
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
use captrs::Capturer;


#[derive(Debug, Serialize, Deserialize)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthBar {
    pub program_name: String,
    pub healthy_color: RGB,
    pub unhealthy_color: RGB,
    pub output_healthy: RGB,
    pub output_unhealthy: RGB,
    pub left_x: u32,
    pub top_y: u32,
    pub right_x: u32,
    pub bottom_y: u32,
    pub max_health_ratio_healthy: f32,
    pub priority: u32,
    pub l2r: bool
}

pub enum ScreenRegion {
    FULL,
    HEALTH(HealthBar)
}

#[derive(Debug, Serialize)]
pub struct Colors {
    pub color_vec: Vec<RGB>
}

pub fn color_thread(screen_id: u32, num_leds: u32, update_interval_ms: u32, reg_reciever: Receiver<ScreenRegion>, colors_tx: Sender<Colors>){
    let mut capt = Capturer::new(screen_id as usize).unwrap();
    let (w, h) = capt.geometry();
    let size = w as u64 * h as u64;

    let mut cur_setting = ScreenRegion::FULL; 

    loop {
        match reg_reciever.try_recv() {
            Ok(reg) => {
                cur_setting = reg;
            },
            _ => {}
        }
        let colors: Colors = match &cur_setting {
            ScreenRegion::FULL => compute_full_screen(num_leds, &mut capt),
            ScreenRegion::HEALTH(h) => compute_health(num_leds, &mut capt, h)
        };

        colors_tx.send(colors);

        thread::sleep(Duration::from_millis(update_interval_ms.into()));
    }

}

fn xy_to_px(x: u32, y: u32, w: u32, h: u32) -> usize {
    if x >= w || y >= h {
        return 0;
    }
    return (x as usize + (y as usize * w as usize));
    //return (y as usize) + (x as usize * h as usize);
}

fn apply_gamma(val: u8) -> u8 {
    const GAMMA8: [u8; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4,
            4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11,
            12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22,
            22, 23, 24, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37,
            38, 39, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58,
            59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85,
            86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104, 105, 107, 109, 110, 112, 114,
            115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137, 138, 140, 142, 144,
            146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175, 177, 180,
            182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
            223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
        ];
    
    return GAMMA8[val as usize];
}

fn apply_gamma_vec(v: &mut Vec<RGB>){
    for i in 0..v.len() {
        v[i].r = apply_gamma(v[i].r);
        v[i].g = apply_gamma(v[i].g);
        v[i].b = apply_gamma(v[i].b);
    }
}

fn compute_full_screen(num_leds: u32, capt: &mut Capturer) -> Colors{
    let full_start = Instant::now();
    const color_thresh: u8 = 32;

    let capt_start = Instant::now();
    let (w, h) = capt.geometry();
    let capt_dur = capt_start.elapsed();

    let ps = capt.capture_frame().unwrap();

    let col_width = w / num_leds;
    let NUM_SAMPLES: u32 = 1000;
    let y_stride = h / NUM_SAMPLES;

    let mut color_vec = Vec::<RGB>::new();

    for col_idx in 0..num_leds {
        let mut tot_r: u64 = 0;
        let mut tot_g: u64 = 0;
        let mut tot_b: u64 = 0;
        let mut tot_px: u64 = 0;

        let base_x = col_width * col_idx;
        let mut cur_x_offset = 0;
        let mut cur_y = 0u32;

        for px_idx in 0..NUM_SAMPLES {
            let idx = xy_to_px(cur_x_offset + base_x, cur_y, w, h);
            let px = ps[idx];
            if !(px.r < color_thresh && px.g < color_thresh && px.b < color_thresh) {
                tot_r += px.r as u64;
                tot_g += px.g as u64;
                tot_b += px.b as u64;
                tot_px += 1;
            }

            cur_x_offset += 12;
            cur_x_offset %= col_width;

            cur_y += y_stride;
        }

        if tot_px == 0 {
            color_vec.push(RGB {r: 0, g: 0, b: 0});
        }
        else {
            let r = (tot_r/tot_px) as u8;
            let g = (tot_g/tot_px) as u8;
            let b = (tot_b/tot_px) as u8;

            color_vec.push(RGB { r, g, b});
        }

        
    }

    apply_gamma_vec(&mut color_vec);
    let full_dur = full_start.elapsed();

    //println!("CAPTURE TOOK {}", capt_dur.as_millis());
    //println!("FULL OP TOOK {}", full_dur.as_millis());
    return Colors{ color_vec: color_vec };
}

fn compute_health(num_leds: u32, capt: &mut Capturer, h: &HealthBar) -> Colors {
    return Colors { color_vec: Vec::new() };
}
