pub mod draw;

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    if s <= 0.0 {
        return (v, v, v);
    }
    let mut i = (h * 6.0).trunc();
    let f = (h * 6.0) - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    i %= 6.0;
    match i as i32 {
        0 => return (v, t, p),
        1 => return (q, v, p),
        2 => return (p, v, t),
        3 => return (p, q, v),
        4 => return (t, p, v),
        5 => return (v, p, q),
        _ => return (0.0, 0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::draw::*;
    use super::hsv_to_rgb;

    use crossterm::{
        cursor::{Hide, Show},
        event::{poll, read, Event, KeyEvent},
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
        },
    };
    use std::{io::stdout, thread, time::Duration};

    #[test]
    fn dvd_icosphere() {
        let mut stdout = stdout();
        let tsize = size().unwrap();
        let mut screen = Screen::new((0.0, -1.0, -1.0));
        screen.camdir = (0.0, 0.0, 0.0);
        execute!(stdout, EnterAlternateScreen, Hide).unwrap();
        enable_raw_mode().unwrap();
        let mut i: f32 = 0.0;
        let size = tsize.1 as f32 / 6.0;
        let mut movement = (5.0, 5.0);
        let mut ballx = tsize.0 as f32 / 2.0;
        let mut bally = tsize.1 as f32;

        let mut hue = 0.0;

        loop {
            screen.clear();
            let colour = hsv_to_rgb(hue, 1.0, 1.0);

            screen.icosphere(
                (ballx, bally, 0.0),
                (0.0, 0.0, i),
                size,
                2,
                (
                    (colour.0 * 255.0) as u8,
                    (colour.1 * 255.0) as u8,
                    (colour.2 * 255.0) as u8,
                ),
                false,
            );

            screen.write();
            if poll(Duration::from_millis(0)).unwrap() {
                let read = read().unwrap();
                if let Event::Key(KeyEvent {
                    code: _,
                    modifiers: _,
                    kind: _,
                    state: _,
                }) = read
                {
                    break;
                }
            }
            i += movement.0;
            ballx += movement.0 / 2.0;
            bally += movement.1 / 2.0;
            if ballx <= size * 1.618 || ballx >= tsize.0 as f32 - size * 1.618 {
                movement = (-movement.0, movement.1);
            }
            if bally <= size * 1.618 || bally >= tsize.1 as f32 * 2.0 - size * 1.618 {
                movement = (movement.0, -movement.1);
            }
            hue += 0.01;
            thread::sleep(Duration::from_millis(70));
        }

        disable_raw_mode().unwrap();
        execute!(stdout, Show, LeaveAlternateScreen).unwrap();
    }

    #[test]
    fn maintest() {
        let mut stdout = stdout();
        let tsize = size().unwrap();
        let mut screen = Screen::new((0.0, -1.0, -1.0));
        screen.camdir = (0.0, 0.0, 0.0);
        execute!(stdout, EnterAlternateScreen, Hide).unwrap();
        enable_raw_mode().unwrap();
        let mut i: f32 = 0.0;
        let size = tsize.1 as f32 / 1.8;

        loop {
            screen.clear();

            // screen.icosphere(
            //     (tsize.0 as f32 / 5.5, tsize.1 as f32 / 1.5, 0.0),
            //     (i, i, 0.0),
            //     size,
            //     1,
            //     (255, 255, 255),
            //     false,
            // );
            // screen.icosphere(
            //     (tsize.0 as f32 / 4.0 * 3.25, tsize.1 as f32 / 1.5, 0.0),
            //     (i, i, 0.0),
            //     size,
            //     2,
            //     (255, 255, 255),
            //     false,
            // );
            // screen.icosphere(
            //     (tsize.0 as f32 / 2.0, tsize.1 as f32 * 1.34, 0.0),
            //     (i, i, 0.0),
            //     size,
            //     3,
            //     (255, 255, 255),
            //     false,
            // );
            screen.pyramid(
                (tsize.0 as f32 / 2.0, tsize.1 as f32, 0.0),
                (25.0, i, 0.0),
                (size, size, size),
                (255, 255, 255),
                false,
            );

            screen.write();
            if poll(Duration::from_millis(0)).unwrap() {
                let read = read().unwrap();
                if let Event::Key(KeyEvent {
                    code: _,
                    modifiers: _,
                    kind: _,
                    state: _,
                }) = read
                {
                    break;
                }
            }
            i += 3.0;
            thread::sleep(Duration::from_millis(70));
        }

        disable_raw_mode().unwrap();
        execute!(stdout, Show, LeaveAlternateScreen).unwrap();
    }
}
