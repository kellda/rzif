use crate::utils::IO;

impl IO {
    pub fn color(&mut self, color: i16, foreground: bool) {
        if color == -2 {
            return;
        }
        if color == -1 {
            if foreground {
                self.buffer("\x1b[39m", false);
            } else {
                self.buffer("\x1b[49m", false);
            }
            return;
        }
        if color < 0 {
            panic!();
        }
        self.buffer(
            &format!(
                "\x1b[{}8;5;{}m",
                if foreground { "3" } else { "4" },
                convert(color)
            ),
            false,
        );
    }
}

fn convert(color: i16) -> i16 {
    let color = [color >> 10 & 0x1f, color >> 5 & 0x1f, color & 0x1f];
    let col = color.iter().map(|&c| col(c)).collect::<Vec<_>>();
    let col_err = color
        .iter()
        .zip(&col)
        .map(|(a, &b)| (a - COLS[b as usize]).abs())
        .sum::<i16>();
    let gray = gray(color.iter().sum::<i16>() / 3);
    let gray_err = color.iter().map(|a| (a - grays(gray)).abs()).sum::<i16>();
    if gray_err < col_err {
        gray + 232
    } else {
        col[2] * 36 + col[1] * 6 + col[0] + 16
    }
}

fn col(color: i16) -> i16 {
    match color {
        0..=5 => 0,
        6..=13 => 1,
        14..=18 => 2,
        19..=23 => 3,
        24..=28 => 4,
        29..=31 => 5,
        _ => unreachable!(),
    }
}

fn gray(color: i16) -> i16 {
    color
        - match color {
            0..=3 => 0,
            4..=10 => 1,
            11..=15 => 2,
            16..=20 => 3,
            21..=25 => 4,
            26..=28 => 5,
            29 | 30 | 31 => color - 23,
            _ => unreachable!(),
        }
}

const COLS: [i16; 6] = [0, 11, 16, 21, 26, 31];

fn grays(index: i16) -> i16 {
    index
        + match index {
            0..=2 => 1,
            3..=8 => 2,
            9..=12 => 3,
            13..=16 => 4,
            17..=20 => 5,
            21..=23 => 6,
            _ => unreachable!(),
        }
}
