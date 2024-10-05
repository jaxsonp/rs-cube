use crossterm::{
    self,
    cursor::MoveTo,
    style::{self, StyledContent, Stylize},
    terminal, QueueableCommand,
};
use nalgebra::{Matrix3, Rotation3, Vector3};

use core::f32;
use std::{
    cmp::{max, min},
    io::{stdout, Write},
    thread,
    time::{Duration, Instant},
};

const TARGET_FPS: f32 = 30.0;
/// Aspect ratio of characters in the terminal (width / height)
const PX_RATIO: f32 = 0.50;

/// Camera field of view
const CAM_FOV: f32 = 70.0;
/// Distance camera is from cube
const CAM_DIST: f32 = 6.0;

/// Ambient lighting direction
const LIGHT_DIRECTION: Vector3<f32> = Vector3::new(-2.0, -4.0, -1.0);

fn main() -> Result<(), std::io::Error> {
    let mut stdout = stdout();

    // terminal sizing
    let mut w: usize = 0;
    let mut h: usize = 0;
    let _ = update_size(&mut w, &mut h);
    eprintln!("size: {w}, {h}");

    let mut frame_buffer: Vec<u8> = Vec::new();
    let mut z_buffer: Vec<f32> = Vec::new();
    frame_buffer.resize(w * h, 0);
    z_buffer.resize(w * h, 0.0);

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
        (0, 2, 1),
        (0, 3, 2),
        (4, 3, 0),
        (4, 7, 3),
        (5, 7, 4),
        (5, 6, 7),
        (1, 6, 5),
        (1, 2, 6),
        (0, 5, 4),
        (0, 1, 5),
        (3, 6, 2),
        (3, 7, 6),
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

    let light_dir: Vector3<f32> = LIGHT_DIRECTION.normalize();

    // target frame duration (30 fps)
    let frame_duration = Duration::from_micros((1000000.0 / TARGET_FPS) as u64);
    let mut t: u64 = 0;
    let mut start_time: Instant;
    loop {
        start_time = Instant::now();

        // checking for term resize
        if t % 6 == 0 {
            let res = update_size(&mut w, &mut h);

            if res.is_ok_and(|changed| changed == true) {
                // screen resized
                frame_buffer.resize(w * h, 0);
                z_buffer.resize(w * h, 0.0);

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

        // rotating cube
        let rot: Rotation3<f32> = Rotation3::new(Vector3::new(0.00702, 0.022003, 0.006));
        for vert in verts.iter_mut() {
            *vert = rot * (*vert);
        }

        // rendering cube
        frame_buffer.fill(0);
        z_buffer.fill(0.0);
        for i in 0..tris.len() {
            let tri = tris[i];

            let (u0, v0, z0) = cam.get_projection(&verts[tri.0]);
            let (u1, v1, z1) = cam.get_projection(&verts[tri.1]);
            let (u2, v2, z2) = cam.get_projection(&verts[tri.2]);

            let normal = (verts[tri.1] - verts[tri.0]).cross(&(verts[tri.2] - verts[tri.0]));
            let light = light_dir.dot(&normal); // (-1, 1)

            // lighting function ((-1, 1) -> [0, 256))
            let color: u8 = (((light / 2.0 + 0.5) * 256.0) as usize).try_into().unwrap();

            // getting coefficients for interpolating z values
            let mat = match Matrix3::new(
                u0 as f32, v0 as f32, 1.0, u1 as f32, v1 as f32, 1.0, u2 as f32, v2 as f32, 1.0,
            )
            .try_inverse()
            {
                Some(m) => m,
                None => {
                    continue;
                }
            };
            let z_coefficients = mat * Vector3::new(z0, z1, z2);

            // getting bounding box of tri
            let u_min = clamp(min(u0, min(u1, u2)), 0, w - 1);
            let u_max = clamp(max(u0, max(u1, u2)), 0, w - 1);
            let v_min = clamp(min(v0, min(v1, v2)), 0, h - 1);
            let v_max = clamp(max(v0, max(v1, v2)), 0, h - 1);

            for v in v_min..=v_max {
                for u in u_min..=u_max {
                    // checking if point is inside tri
                    if !tri_edge_function((u0, v0), (u1, v1), (u, v))
                        || !tri_edge_function((u1, v1), (u2, v2), (u, v))
                        || !tri_edge_function((u2, v2), (u0, v0), (u, v))
                    {
                        continue;
                    }

                    let buf_index = v * w + u;

                    // checking z buffer
                    let z = z_coefficients.x * (u as f32)
                        + z_coefficients.y * (v as f32)
                        + z_coefficients.z;
                    if z < z_buffer[buf_index] {
                        continue;
                    }
                    z_buffer[buf_index] = z;

                    frame_buffer[buf_index] = color;
                }
            }
        }

        // drawing framebuffer
        for i in 0..(frame_buffer.len()) {
            if i % w == 0 && i != 0 {
                stdout.queue(style::Print("\r\n"))?;
            }
            let c = get_char_from_val(frame_buffer[i]);
            stdout.queue(style::Print(c))?;
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
    pub pos: Vector3<f32>,
    pub a: Vector3<f32>,
    pub b: Vector3<f32>,
    pub c: Vector3<f32>,
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

/// Function that turns a color value from 0-255 into a char to print
fn get_char_from_val(val: u8) -> char {
    match val >> 5 {
        7 => '@',
        6 => '0',
        5 => 'O',
        4 => '+',
        3 => '=',
        2 => ':',
        1 => '.',
        _ => ' ',
    }
}

/// Util function for bounding value
fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    if val > max {
        return max;
    } else if val < min {
        return min;
    }
    return val;
}

/// Util function for triangle rasterization, returns true if the points are in clockwise order (a -> b -> c)
fn tri_edge_function(
    (ax, ay): (usize, usize),
    (bx, by): (usize, usize),
    (cx, cy): (usize, usize),
) -> bool {
    let ax = ax as isize;
    let ay = ay as isize;
    let bx = bx as isize;
    let by = by as isize;
    let cx = cx as isize;
    let cy = cy as isize;
    return (bx - ax) * (cy - ay) - (by - ay) * (cx - ax) >= 0;
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
