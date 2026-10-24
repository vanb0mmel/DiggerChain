use std::fmt::Write as _;
use std::io::{self, Write};

use super::art::{BORDER, PANEL, Rgb, TEXT, VOID};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    ch: char,
    fg: Rgb,
    bg: Rgb,
}

pub struct Screen {
    pub width: i32,
    pub height: i32,
    cells: Vec<Cell>,
}

impl Screen {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width: width as i32,
            height: height as i32,
            cells: vec![
                Cell {
                    ch: ' ',
                    fg: TEXT,
                    bg: VOID
                };
                width as usize * height as usize
            ],
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && x < self.width && y < self.height)
            .then(|| (y * self.width + x) as usize)
    }

    pub fn set(&mut self, x: i32, y: i32, ch: char, fg: Rgb, bg: Rgb) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = Cell { ch, fg, bg };
        }
    }

    pub fn text(&mut self, x: i32, y: i32, text: &str, color: Rgb) {
        for (offset, ch) in text.chars().enumerate() {
            if let Some(i) = self.index(x + offset as i32, y) {
                self.cells[i].ch = ch;
                self.cells[i].fg = color;
            }
        }
    }

    pub fn clipped(&mut self, x: i32, y: i32, width: i32, text: &str, color: Rgb) {
        let text: String = text.chars().take(width.max(0) as usize).collect();
        self.text(x, y, &text, color);
    }

    pub fn fill(&mut self, x: i32, y: i32, width: i32, height: i32, color: Rgb) {
        for row in y..y + height {
            for col in x..x + width {
                self.set(col, row, ' ', TEXT, color);
            }
        }
    }

    pub fn frame(&mut self, x: i32, y: i32, width: i32, height: i32, title: &str) {
        self.fill(x, y, width, height, PANEL);
        for col in x + 1..x + width - 1 {
            self.set(col, y, '─', BORDER, PANEL);
            self.set(col, y + height - 1, '─', BORDER, PANEL);
        }
        for row in y + 1..y + height - 1 {
            self.set(x, row, '│', BORDER, PANEL);
            self.set(x + width - 1, row, '│', BORDER, PANEL);
        }
        for (col, row, ch) in [
            (x, y, '╭'),
            (x + width - 1, y, '╮'),
            (x, y + height - 1, '╰'),
            (x + width - 1, y + height - 1, '╯'),
        ] {
            self.set(col, row, ch, BORDER, PANEL);
        }
        self.clipped(x + 2, y, width - 4, &format!(" {title} "), TEXT);
    }

    pub fn wrap(&mut self, x: i32, mut y: i32, width: i32, lines: i32, text: &str, color: Rgb) {
        let mut line = String::new();
        let end = y + lines;
        for word in text.split_whitespace() {
            if !line.is_empty() && line.chars().count() + word.chars().count() + 1 > width as usize
            {
                self.clipped(x, y, width, &line, color);
                y += 1;
                line.clear();
                if y >= end {
                    return;
                }
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if y < end {
            self.clipped(x, y, width, &line, color);
        }
    }

    pub fn flush(self, previous: &mut Option<Self>, output: &mut impl Write) -> io::Result<()> {
        let old = previous
            .as_ref()
            .filter(|old| old.width == self.width && old.height == self.height);
        let mut buffer = String::new();
        let mut foreground = None;
        let mut background = None;
        for y in 0..self.height {
            let mut adjacent = false;
            for x in 0..self.width {
                let i = (y * self.width + x) as usize;
                let cell = self.cells[i];
                if old.is_some_and(|old| old.cells[i] == cell) {
                    adjacent = false;
                    continue;
                }
                if !adjacent {
                    let _ = write!(buffer, "\x1b[{};{}H", y + 1, x + 1);
                }
                if foreground != Some(cell.fg) {
                    let Rgb(r, g, b) = cell.fg;
                    let _ = write!(buffer, "\x1b[38;2;{r};{g};{b}m");
                    foreground = Some(cell.fg);
                }
                if background != Some(cell.bg) {
                    let Rgb(r, g, b) = cell.bg;
                    let _ = write!(buffer, "\x1b[48;2;{r};{g};{b}m");
                    background = Some(cell.bg);
                }
                buffer.push(cell.ch);
                adjacent = true;
            }
        }
        if !buffer.is_empty() {
            output.write_all(b"\x1b[?2026h")?;
            output.write_all(buffer.as_bytes())?;
            output.write_all(b"\x1b[0m\x1b[?2026l")?;
            output.flush()?;
        }
        *previous = Some(self);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unchanged_frame_does_not_write_to_the_terminal() {
        let mut previous = None;
        Screen::new(10, 4)
            .flush(&mut previous, &mut Vec::new())
            .unwrap();
        let mut output = Vec::new();
        Screen::new(10, 4)
            .flush(&mut previous, &mut output)
            .unwrap();
        assert!(output.is_empty());
    }
}
