use crate::buffer::Cell;
use crossterm::style::Color;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::cell::RefCell;
use std::io::{Read, Write};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use vte::{Params, Perform};

/// A 2D grid of terminal cells with cursor tracking.
#[derive(Clone)]
pub struct TerminalBuffer {
    cells: Vec<Vec<Cell>>,
    rows: usize,
    cols: usize,
    cursor_row: usize,
    cursor_col: usize,
    cursor_visible: bool,
    // Current SGR state for new characters
    current_fg: Color,
    current_bg: Color,
    current_bold: bool,
    current_italic: bool,
    current_underline: bool,
    current_dim: bool,
}

impl TerminalBuffer {
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![vec![Cell::default(); cols]; rows];
        Self {
            cells,
            rows,
            cols,
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            current_bold: false,
            current_italic: false,
            current_underline: false,
            current_dim: false,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn cursor_row(&self) -> usize {
        self.cursor_row
    }

    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    pub fn cells(&self) -> &Vec<Vec<Cell>> {
        &self.cells
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.cells.get(row).and_then(|r| r.get(col))
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        if row < self.rows && col < self.cols {
            self.cells[row][col] = cell;
        }
    }

    pub fn write_char(&mut self, ch: char) {
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            let cell = Cell::styled(
                ch,
                self.current_fg,
                self.current_bg,
                self.current_bold,
                self.current_italic,
                self.current_underline,
                self.current_dim,
            );
            self.cells[self.cursor_row][self.cursor_col] = cell;
            self.cursor_col += 1;

            // Wrap to next line if we exceed columns
            if self.cursor_col >= self.cols {
                self.cursor_col = 0;
                self.cursor_row += 1;
                // Scroll if we exceed rows
                if self.cursor_row >= self.rows {
                    self.scroll_up();
                    self.cursor_row = self.rows - 1;
                }
            }
        }
    }

    pub fn move_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows.saturating_sub(1));
        self.cursor_col = col.min(self.cols.saturating_sub(1));
    }

    pub fn move_cursor_relative(&mut self, row_delta: isize, col_delta: isize) {
        let new_row = (self.cursor_row as isize + row_delta).max(0) as usize;
        let new_col = (self.cursor_col as isize + col_delta).max(0) as usize;
        self.move_cursor(new_row, new_col);
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
    }

    pub fn clear_line(&mut self, row: usize) {
        if row < self.rows {
            for col in 0..self.cols {
                self.cells[row][col] = Cell::default();
            }
        }
    }

    pub fn clear_line_from_cursor(&mut self) {
        let row = self.cursor_row;
        if row < self.rows {
            for col in self.cursor_col..self.cols {
                self.cells[row][col] = Cell::default();
            }
        }
    }

    pub fn clear_line_to_cursor(&mut self) {
        let row = self.cursor_row;
        if row < self.rows {
            for col in 0..=self.cursor_col.min(self.cols - 1) {
                self.cells[row][col] = Cell::default();
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.cells.remove(0);
        self.cells.push(vec![Cell::default(); self.cols]);
    }

    pub fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    pub fn newline(&mut self) {
        self.cursor_row += 1;
        if self.cursor_row >= self.rows {
            self.scroll_up();
            self.cursor_row = self.rows - 1;
        }
    }

    pub fn tab(&mut self) {
        // Tab stops at multiples of 8
        let next_stop = ((self.cursor_col / 8) + 1) * 8;
        self.cursor_col = next_stop.min(self.cols - 1);
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub fn set_sgr(&mut self, fg: Color, bg: Color, bold: bool, italic: bool, underline: bool, dim: bool) {
        self.current_fg = fg;
        self.current_bg = bg;
        self.current_bold = bold;
        self.current_italic = italic;
        self.current_underline = underline;
        self.current_dim = dim;
    }

    pub fn reset_sgr(&mut self) {
        self.current_fg = Color::Reset;
        self.current_bg = Color::Reset;
        self.current_bold = false;
        self.current_italic = false;
        self.current_underline = false;
        self.current_dim = false;
    }
}

/// Messages sent from PTY reader thread to main thread.
enum PtyMessage {
    Output(Vec<u8>),
    Exited,
}

/// Handle to a running PTY process.
#[derive(Clone)]
pub struct TerminalHandle {
    inner: Rc<RefCell<TerminalHandleInner>>,
}

struct TerminalHandleInner {
    buffer: TerminalBuffer,
    parser: vte::Parser,
    pty_pair: Option<PtyPair>,
    output_rx: Option<Receiver<PtyMessage>>,
    is_started: bool,
    is_exited: bool,
}

struct PtyPair {
    writer: Box<dyn Write + Send>,
    _reader_thread: thread::JoinHandle<()>,
}

impl TerminalHandle {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            inner: Rc::new(RefCell::new(TerminalHandleInner {
                buffer: TerminalBuffer::new(rows, cols),
                parser: vte::Parser::new(),
                pty_pair: None,
                output_rx: None,
                is_started: false,
                is_exited: false,
            })),
        }
    }

    pub fn is_started(&self) -> bool {
        self.inner.borrow().is_started
    }

    pub fn is_exited(&self) -> bool {
        self.inner.borrow().is_exited
    }

    pub fn spawn(&self, command: &str, args: &[&str], cols: usize, rows: usize) -> Result<(), String> {
        let mut inner = self.inner.borrow_mut();

        if inner.is_started {
            return Err("Terminal already started".to_string());
        }

        let pty_system = native_pty_system();

        let pty_pair = pty_system
            .openpty(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        let mut child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        // Spawn reader thread
        let (output_tx, output_rx) = mpsc::channel();
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - process exited
                        let _ = output_tx.send(PtyMessage::Exited);
                        break;
                    }
                    Ok(n) => {
                        if output_tx.send(PtyMessage::Output(buf[..n].to_vec())).is_err() {
                            // Main thread dropped the receiver
                            break;
                        }
                    }
                    Err(_) => {
                        let _ = output_tx.send(PtyMessage::Exited);
                        break;
                    }
                }
            }
            // Reap the child process
            let _ = child.wait();
        });

        let writer = pty_pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        inner.pty_pair = Some(PtyPair {
            writer,
            _reader_thread: reader_thread,
        });
        inner.output_rx = Some(output_rx);
        inner.is_started = true;
        inner.buffer = TerminalBuffer::new(rows, cols);

        Ok(())
    }

    pub fn poll(&self) {
        // Collect messages first without holding mutable borrow
        let messages: Vec<PtyMessage> = {
            let inner = self.inner.borrow();

            if !inner.is_started || inner.is_exited {
                return;
            }

            let Some(ref rx) = inner.output_rx else {
                return;
            };

            // Drain all available messages (non-blocking)
            let mut msgs = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                msgs.push(msg);
            }
            msgs
        };

        // Now process messages with mutable borrow
        for msg in messages {
            match msg {
                PtyMessage::Output(data) => {
                    let mut inner = self.inner.borrow_mut();
                    // Process the data through the parser
                    // We need to temporarily access both buffer and parser
                    // Split the borrow by using raw pointers (safe because we know the lifetimes)
                    let buffer_ptr = &mut inner.buffer as *mut TerminalBuffer;
                    let parser_ptr = &mut inner.parser as *mut vte::Parser;

                    unsafe {
                        let mut performer = TerminalPerformer {
                            buffer: &mut *buffer_ptr,
                        };
                        (*parser_ptr).advance(&mut performer, &data);
                    }
                }
                PtyMessage::Exited => {
                    let mut inner = self.inner.borrow_mut();
                    inner.is_exited = true;
                    return;
                }
            }
        }
    }

    pub fn send_input(&self, data: &[u8]) -> Result<(), String> {
        let mut inner = self.inner.borrow_mut();

        if !inner.is_started || inner.is_exited {
            return Err("Terminal not running".to_string());
        }

        if let Some(ref mut pty_pair) = inner.pty_pair {
            pty_pair.writer.write_all(data)
                .map_err(|e| format!("Failed to write to PTY: {}", e))?;
            pty_pair.writer.flush()
                .map_err(|e| format!("Failed to flush PTY: {}", e))?;
        }

        Ok(())
    }

    pub fn get_buffer(&self) -> TerminalBuffer {
        self.inner.borrow().buffer.clone()
    }

    pub fn resize(&self, rows: usize, cols: usize) -> Result<(), String> {
        let mut inner = self.inner.borrow_mut();

        if !inner.is_started {
            return Ok(());
        }

        if let Some(ref mut _pty_pair) = inner.pty_pair {
            // Note: portable-pty doesn't expose resize on the writer, so we skip it for now
            // In a full implementation, we'd need to store the master PTY handle
        }

        inner.buffer = TerminalBuffer::new(rows, cols);
        Ok(())
    }
}

/// Implements vte::Perform to handle ANSI escape sequences.
struct TerminalPerformer<'a> {
    buffer: &'a mut TerminalBuffer,
}

impl<'a> Perform for TerminalPerformer<'a> {
    fn print(&mut self, c: char) {
        self.buffer.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                // Line feed
                self.buffer.newline();
            }
            b'\r' => {
                // Carriage return
                self.buffer.carriage_return();
            }
            b'\t' => {
                // Tab
                self.buffer.tab();
            }
            0x08 => {
                // Backspace
                self.buffer.backspace();
            }
            0x07 => {
                // Bell - ignore for now
            }
            _ => {
                // Ignore other control characters
            }
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {
        // DCS sequences - not needed for MVP
    }

    fn put(&mut self, _byte: u8) {
        // DCS sequences - not needed for MVP
    }

    fn unhook(&mut self) {
        // DCS sequences - not needed for MVP
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences (like setting title) - not needed for MVP
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'H' | 'f' => {
                // Cursor Position (CUP): ESC[{row};{col}H
                let row = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(1);
                let col = params.iter().nth(1).and_then(|p| p.first().copied()).unwrap_or(1);
                self.buffer.move_cursor(
                    (row as usize).saturating_sub(1),
                    (col as usize).saturating_sub(1),
                );
            }
            'A' => {
                // Cursor Up (CUU): ESC[{n}A
                let n = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(1);
                self.buffer.move_cursor_relative(-(n as isize), 0);
            }
            'B' => {
                // Cursor Down (CUD): ESC[{n}B
                let n = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(1);
                self.buffer.move_cursor_relative(n as isize, 0);
            }
            'C' => {
                // Cursor Forward (CUF): ESC[{n}C
                let n = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(1);
                self.buffer.move_cursor_relative(0, n as isize);
            }
            'D' => {
                // Cursor Back (CUB): ESC[{n}D
                let n = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(1);
                self.buffer.move_cursor_relative(0, -(n as isize));
            }
            'J' => {
                // Erase in Display (ED): ESC[{n}J
                let n = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(0);
                match n {
                    0 => {
                        // Clear from cursor to end of screen
                        self.buffer.clear_line_from_cursor();
                        let row = self.buffer.cursor_row();
                        for r in (row + 1)..self.buffer.rows() {
                            self.buffer.clear_line(r);
                        }
                    }
                    1 => {
                        // Clear from cursor to beginning of screen
                        self.buffer.clear_line_to_cursor();
                        let row = self.buffer.cursor_row();
                        for r in 0..row {
                            self.buffer.clear_line(r);
                        }
                    }
                    2 => {
                        // Clear entire screen
                        self.buffer.clear_screen();
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line (EL): ESC[{n}K
                let n = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(0);
                match n {
                    0 => self.buffer.clear_line_from_cursor(),
                    1 => self.buffer.clear_line_to_cursor(),
                    2 => self.buffer.clear_line(self.buffer.cursor_row()),
                    _ => {}
                }
            }
            'm' => {
                // SGR - Select Graphic Rendition: ESC[{n}m
                let mut fg = self.buffer.current_fg;
                let mut bg = self.buffer.current_bg;
                let mut bold = self.buffer.current_bold;
                let mut italic = self.buffer.current_italic;
                let mut underline = self.buffer.current_underline;
                let mut dim = self.buffer.current_dim;

                let mut iter = params.iter();
                while let Some(param) = iter.next() {
                    let n = param.first().copied().unwrap_or(0);
                    match n {
                        0 => {
                            // Reset
                            fg = Color::Reset;
                            bg = Color::Reset;
                            bold = false;
                            italic = false;
                            underline = false;
                            dim = false;
                        }
                        1 => bold = true,
                        2 => dim = true,
                        3 => italic = true,
                        4 => underline = true,
                        22 => {
                            bold = false;
                            dim = false;
                        }
                        23 => italic = false,
                        24 => underline = false,
                        // Foreground colors (30-37)
                        30 => fg = Color::Black,
                        31 => fg = Color::DarkRed,
                        32 => fg = Color::DarkGreen,
                        33 => fg = Color::DarkYellow,
                        34 => fg = Color::DarkBlue,
                        35 => fg = Color::DarkMagenta,
                        36 => fg = Color::DarkCyan,
                        37 => fg = Color::Grey,
                        38 => {
                            // 256-color or RGB
                            if let Some(next_param) = iter.next() {
                                let mode = next_param.first().copied().unwrap_or(0);
                                if mode == 5 {
                                    // 256-color: ESC[38;5;{n}m
                                    if let Some(color_param) = iter.next() {
                                        let color_idx = color_param.first().copied().unwrap_or(0);
                                        fg = Color::AnsiValue(color_idx as u8);
                                    }
                                } else if mode == 2 {
                                    // RGB: ESC[38;2;{r};{g};{b}m
                                    let r = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                    let g = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                    let b = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                    fg = Color::Rgb { r, g, b };
                                }
                            }
                        }
                        39 => fg = Color::Reset,
                        // Background colors (40-47)
                        40 => bg = Color::Black,
                        41 => bg = Color::DarkRed,
                        42 => bg = Color::DarkGreen,
                        43 => bg = Color::DarkYellow,
                        44 => bg = Color::DarkBlue,
                        45 => bg = Color::DarkMagenta,
                        46 => bg = Color::DarkCyan,
                        47 => bg = Color::Grey,
                        48 => {
                            // 256-color or RGB
                            if let Some(next_param) = iter.next() {
                                let mode = next_param.first().copied().unwrap_or(0);
                                if mode == 5 {
                                    // 256-color: ESC[48;5;{n}m
                                    if let Some(color_param) = iter.next() {
                                        let color_idx = color_param.first().copied().unwrap_or(0);
                                        bg = Color::AnsiValue(color_idx as u8);
                                    }
                                } else if mode == 2 {
                                    // RGB: ESC[48;2;{r};{g};{b}m
                                    let r = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                    let g = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                    let b = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                    bg = Color::Rgb { r, g, b };
                                }
                            }
                        }
                        49 => bg = Color::Reset,
                        // Bright foreground colors (90-97)
                        90 => fg = Color::DarkGrey,
                        91 => fg = Color::Red,
                        92 => fg = Color::Green,
                        93 => fg = Color::Yellow,
                        94 => fg = Color::Blue,
                        95 => fg = Color::Magenta,
                        96 => fg = Color::Cyan,
                        97 => fg = Color::White,
                        // Bright background colors (100-107)
                        100 => bg = Color::DarkGrey,
                        101 => bg = Color::Red,
                        102 => bg = Color::Green,
                        103 => bg = Color::Yellow,
                        104 => bg = Color::Blue,
                        105 => bg = Color::Magenta,
                        106 => bg = Color::Cyan,
                        107 => bg = Color::White,
                        _ => {}
                    }
                }

                self.buffer.set_sgr(fg, bg, bold, italic, underline, dim);
            }
            'h' => {
                // Set Mode
                if let Some(param) = params.iter().next() {
                    let n = param.first().copied().unwrap_or(0);
                    if n == 25 {
                        // Show cursor
                        self.buffer.set_cursor_visible(true);
                    }
                }
            }
            'l' => {
                // Reset Mode
                if let Some(param) = params.iter().next() {
                    let n = param.first().copied().unwrap_or(0);
                    if n == 25 {
                        // Hide cursor
                        self.buffer.set_cursor_visible(false);
                    }
                }
            }
            _ => {
                // Ignore other CSI sequences
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences - not needed for MVP
    }
}
