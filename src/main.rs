use std::{
    io::{stdout, Write},
    process, thread,
    time::{Duration, Instant},
};
//use termion::{clear, cursor, raw::IntoRawMode, terminal_size};
use crossterm::{
    self,
    cursor::MoveTo,
    style,
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};

struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

const TARGET_FPS: f32 = 2.0;

fn main() -> Result<(), std::io::Error> {
    // io config
    let mut stdout = stdout();

    // terminal width and height
    let mut w: usize = 0;
    let mut h: usize = 0;

    // framebuffer
    let mut fb: Vec<char> = Vec::new();

    fb.fill('*');

    // target frame duration (30 fps)
    let frame_duration = Duration::from_micros((1000000.0 / TARGET_FPS) as u64);

    // loop
    let mut t: u64 = 0;
    loop {
        let start_time = Instant::now();

        if t % 6 == 0 {
            check_resize(&mut w, &mut h, &mut fb).unwrap();
        }

        // drawing framebuffer
        //write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;
        //stdout.queue(Clear(ClearType::All))?;
        stdout.queue(MoveTo(0, 0))?;
        for i in 0..(w * h) {
            if i % w == 0 && i != 0 {
                stdout.queue(style::Print("\r\n"))?;
            }
            stdout.queue(style::Print(fb[i]))?;
        }

        stdout.flush()?;

        t += 1;
        thread::sleep(frame_duration - start_time.elapsed());
    }

    Ok(())
}

fn check_resize(w: &mut usize, h: &mut usize, fb: &mut Vec<char>) -> Result<(), ()> {
    if let Ok(size) = terminal::size() {
        let new_w = size.0 as usize;
        let new_h = size.1 as usize;
        if new_w != *w || new_h != *h {
            *w = new_w;
            *h = new_h;
            eprint!("enw size");
            fb.resize((*w * *h) as usize, '8');
        }
        return Ok(());
    } else {
        eprintln!("Failed to get terminal size");
        return Err(());
    }
}
