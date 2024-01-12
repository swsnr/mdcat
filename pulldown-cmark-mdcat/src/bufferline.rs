use std::{
    io::{stdout, Write},
    mem,
};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    queue,
};
use ratatui::layout::Rect;

use crate::markdown_widget::Offset;

#[derive(Debug, Clone)]
pub enum BufferLine {
    Line(Vec<u8>),
    Image { data: Vec<u8>, height: u16 },
}

impl BufferLine {
    pub fn height(&self) -> u16 {
        match self {
            BufferLine::Line(_) => 1,
            BufferLine::Image { height, .. } => *height,
        }
    }

    pub fn data(&self) -> &[u8] {
        match self {
            BufferLine::Line(v) => v,
            BufferLine::Image { data, .. } => data,
        }
    }
}

pub fn render_buffer_lines(buffer_lines: &[BufferLine], area: Rect, offset: &mut Offset) {
    if buffer_lines.is_empty() {
        return;
    }
    let accumulative_end: Vec<u16> = {
        let mut v = Vec::new();
        let mut a = 0;
        for x in buffer_lines {
            a += x.height();
            v.push(a)
        }
        v
    };
    let mut start = match accumulative_end.binary_search_by(|x| x.cmp(&offset.y)) {
        Ok(a) => a + 1,
        Err(0) => 0,
        Err(a) => a + 1,
    };
    if start >= buffer_lines.len() {
        start = buffer_lines.len() - 1;
    }

    offset.y = accumulative_end[start] - buffer_lines[start].height();

    let mut stdout = stdout().lock();
    queue!(&mut stdout, SavePosition).unwrap();

    let mut used_lines: u16 = 0;

    for i in start..buffer_lines.len() {
        if used_lines + buffer_lines[i].height() > area.height {
            break;
        } else {
            queue!(&mut stdout, MoveTo(area.x, area.y + used_lines)).unwrap();
            stdout.write_all(buffer_lines[i].data()).unwrap();
            used_lines += buffer_lines[i].height();
        }
    }

    queue!(&mut stdout, RestorePosition).unwrap();
    stdout.flush().unwrap();
}

pub struct BufferLines {
    pub lines: Vec<BufferLine>,
    pub current_buffer: Vec<u8>,
}

impl BufferLines {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_buffer: Vec::new(),
        }
    }

    pub fn finish(mut self) -> Vec<BufferLine> {
        if !self.current_buffer.is_empty() {
            self.lines.push(BufferLine::Line(self.current_buffer));
        }
        self.lines
    }

    pub fn writeln_buffer(&mut self) {
        let old = mem::replace(&mut self.current_buffer, Vec::new());
        self.lines.push(BufferLine::Line(old));
    }

    pub fn write_image(&mut self, height: u16) {
        let data = mem::replace(&mut self.current_buffer, Vec::new());
        self.lines.push(BufferLine::Image { data, height });
    }
}

impl Write for BufferLines {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.current_buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.current_buffer.flush()
    }
}
