use std::{
    io::{stdout, Write},
    process, thread,
    time::{Duration, Instant},
};
use termion::{clear, cursor, terminal_size};

const TARGET_FPS: f32 = 1.0;

fn main() -> Result<(), std::io::Error> {
    // io config
    let mut stdout = stdout();

    // terminal width and height
    let mut w: usize = 0;
    let mut h: usize = 0;

    // framebuffer
    let mut fb: Vec<char> = Vec::new();

    if handle_resize(&mut w, &mut h, &mut fb).is_err() {
        process::exit(1);
    }

    fb.fill('*');

    // target frame duration (30 fps)
    let frame_duration = Duration::from_micros((1000000.0 / TARGET_FPS) as u64);

    loop {
        let start_time = Instant::now();

        // drawing framebuffer
        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;
        for i in 0..(w * h) {
            if i % w == 0 && i != 0 {
                write!(stdout, "\r\n")?;
            }
            write!(stdout, "{}", fb[i])?;
        }

        stdout.flush()?;

        thread::sleep(frame_duration - start_time.elapsed());
    }

    Ok(())
}

fn handle_resize(w: &mut usize, h: &mut usize, fb: &mut Vec<char>) -> Result<(), ()> {
    if let Ok(size) = terminal_size() {
        *w = size.0 as usize;
        *h = size.1 as usize;

        fb.resize((*w * *h) as usize, ' ');

        return Ok(());
    } else {
        eprintln!("Failed to get terminal size");
        return Err(());
    }
}
