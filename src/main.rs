use crossterm::{self, cursor::MoveTo, style, terminal, QueueableCommand};
use nalgebra::{Matrix3, Vector3};

use core::f32;
use std::{
    io::{stdout, Write},
    thread,
    time::{Duration, Instant},
};

const TARGET_FPS: f32 = 30.0;
/// Aspect ratio of characters in the terminal (width / height)
const PX_RATIO: f32 = 0.50;

/// Camera field of view
const CAM_FOV: f32 = 50.0;
/// Distance camera is from cube
const CAM_DIST: f32 = 10.0;

fn main() -> Result<(), std::io::Error> {
    let mut stdout = stdout();

    // terminal sizing
    let mut w: usize = 0;
    let mut h: usize = 0;
    let _ = update_size(&mut w, &mut h);
    eprintln!("size: {w}, {h}");

    let mut frame_buffer: Vec<char> = Vec::new();
    let mut z_buffer: Vec<f32> = Vec::new();
    frame_buffer.resize(w * h, ' ');
    z_buffer.resize(w * h, f32::INFINITY);

    let mut verts: [Vector3<f32>; 8] = [
        Vector3::new(0.5, 0.5, 0.5),
        Vector3::new(-0.5, 0.5, 0.5),
        Vector3::new(-0.5, -0.5, 0.5),
        Vector3::new(0.5, -0.5, 0.5),
        Vector3::new(0.5, 0.5, -0.5),
        Vector3::new(-0.5, 0.5, -0.5),
        Vector3::new(-0.5, -0.5, -0.5),
        Vector3::new(0.5, -0.5, -0.5),
    ];
    let tris: [(usize, usize, usize); 12] = [
        (0, 1, 2),
        (0, 2, 3),
        (4, 6, 5),
        (4, 7, 6),
        (3, 2, 6),
        (3, 6, 7),
        (0, 5, 1),
        (0, 4, 5),
        (0, 7, 3),
        (0, 4, 7),
        (1, 6, 2),
        (1, 5, 6),
    ];

    let fov = CAM_FOV.to_radians();
    let mut cam = Camera {
        pos: Vector3::new(0.0, 0.0, CAM_DIST),
        a: Vector3::new(PX_RATIO, 0.0, 0.0),
        b: Vector3::new(0.0, -1.0, 0.0),
        c: Vector3::new(
            -(w as f32 * PX_RATIO) / 2.0,
            (h as f32) / 2.0,
            -(w as f32 / (2.0 * f32::tan(fov / 2.0))),
        ),
    };

    // target frame duration (30 fps)
    let frame_duration = Duration::from_micros((1000000.0 / TARGET_FPS) as u64);
    let mut t: u64 = 0;
    loop {
        let start_time = Instant::now();

        if t % 6 == 0 {
            let res = update_size(&mut w, &mut h);

            if res.is_ok_and(|changed| changed == true) {
                // screen resized:
                frame_buffer.resize(w * h, ' ');
                z_buffer.resize(w * h, f32::INFINITY);

                cam.c = Vector3::new(
                    -(w as f32 * PX_RATIO) / 2.0,
                    (h as f32) / 2.0,
                    -(w as f32 / (2.0 * f32::tan(fov / 2.0))),
                );
            } else if res.is_err() {
                eprintln!("Failed to get terminal size");
                std::process::exit(1);
            }
        }

        frame_buffer.fill(' ');
        for vert in verts {
            let (u, v, _) = cam.get_projection(&vert);
            if (u >= w || v >= h) {
                continue;
            }
            frame_buffer[v * w + u] = '#';
        }

        // drawing framebuffer
        for i in 0..(frame_buffer.len()) {
            if i % w == 0 && i != 0 {
                stdout.queue(style::Print("\r\n"))?;
            }
            stdout.queue(style::Print(frame_buffer[i]))?;
        }

        stdout.flush()?;
        stdout.queue(MoveTo(0, 0))?;

        t += 1;
        let elapsed = start_time.elapsed();
        if frame_duration > elapsed {
            thread::sleep(frame_duration - start_time.elapsed());
        }
        //return Ok(());
    }
}
struct Camera {
    pos: Vector3<f32>,
    a: Vector3<f32>,
    b: Vector3<f32>,
    c: Vector3<f32>,
}

impl Camera {
    fn get_projection(&self, point: &Vector3<f32>) -> (usize, usize, f32) {
        let mat: Matrix3<f32> = Matrix3::from_columns(&[self.a, self.b, self.c])
            .try_inverse()
            .unwrap();
        //eprintln!("mat: {mat}");
        let res = mat * (point - self.pos);
        let z = res.z;
        let u = res.x / z;
        let v = res.y / z;
        return (u as usize, v as usize, z);
    }
}

/// Returns a result, containing a boolean representing if the terminal has changed size
fn update_size(w: &mut usize, h: &mut usize) -> Result<bool, ()> {
    if let Ok(size) = terminal::size() {
        let new_w = size.0 as usize;
        let new_h = size.1 as usize;

        if new_w == *w && new_h == *h {
            return Ok(false);
        }

        *w = new_w;
        *h = new_h;
        return Ok(true);
    } else {
        return Err(());
    }
}

/// Calculates a cube scale factor to fit it in the terminal
fn calculate_cube_size(w: usize, h: usize) -> f32 {
    let min_dim = f32::min(w as f32 * PX_RATIO, h as f32);
    return 2.0;
    //_dim / 20.0;
}
