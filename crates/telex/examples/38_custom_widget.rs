//! Example 38: Custom Widget — Game of Life
//!
//! Conway's Game of Life implemented as a custom Widget.
//! Demonstrates the Widget trait escape hatch for character-cell rendering.
//!
//! Run with: `cargo run -p telex-tui --example 38_custom_widget`

use crossterm::event::KeyCode;
use crossterm::style::Color;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use telex::buffer::{Buffer, Rect};
use telex::prelude::*;
use telex::widget::Widget;

telex::require_api!(0, 2);

fn main() {
    telex::run(App).unwrap();
}

const GRID_W: usize = 40;
const GRID_H: usize = 20;

#[derive(Clone)]
struct GameOfLife {
    grid: Vec<Vec<bool>>,
}

impl GameOfLife {
    fn new() -> Self {
        Self {
            grid: vec![vec![false; GRID_W]; GRID_H],
        }
    }

    fn randomize(&mut self) {
        // Simple pseudo-random using a seed based on grid state
        let mut seed: u64 = 12345;
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                *cell = seed % 3 == 0; // ~33% alive
            }
        }
    }

    fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
    }

    fn step(&mut self) {
        let mut next = vec![vec![false; GRID_W]; GRID_H];
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let neighbors = self.count_neighbors(x, y);
                next[y][x] = match (self.grid[y][x], neighbors) {
                    (true, 2..=3) => true,  // survive
                    (false, 3) => true,     // birth
                    _ => false,             // die
                };
            }
        }
        self.grid = next;
    }

    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        for dy in [-1i32, 0, 1] {
            for dx in [-1i32, 0, 1] {
                if dy == 0 && dx == 0 {
                    continue;
                }
                let nx = (x as i32 + dx).rem_euclid(GRID_W as i32) as usize;
                let ny = (y as i32 + dy).rem_euclid(GRID_H as i32) as usize;
                if self.grid[ny][nx] {
                    count += 1;
                }
            }
        }
        count
    }

    fn alive_count(&self) -> usize {
        self.grid.iter().flat_map(|r| r.iter()).filter(|&&c| c).count()
    }
}

impl Widget for GameOfLife {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        for y in 0..GRID_H.min(area.height as usize) {
            for x in 0..GRID_W.min(area.width as usize) {
                let ch = if self.grid[y][x] { '\u{2588}' } else { '\u{00B7}' };
                let fg = if self.grid[y][x] { Color::Green } else { Color::DarkGrey };
                buf.set(area.x + x as u16, area.y + y as u16, ch, fg, Color::Reset);
            }
        }
    }

    fn height_hint(&self, _width: u16) -> Option<u16> {
        Some(GRID_H as u16)
    }

    fn width_hint(&self) -> Option<u16> {
        Some(GRID_W as u16)
    }
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let game: State<GameOfLife> = state!(cx, || {
            let mut g = GameOfLife::new();
            g.randomize();
            g
        });
        let generation = state!(cx, || 0u64);
        let playing = state!(cx, || false);

        // Auto-step when playing
        interval!(cx, Duration::from_millis(150), with!(playing, game, generation => move || {
            if playing.get() {
                game.update(|g| g.step());
                generation.update(|n| *n += 1);
            }
        }));

        let widget = Rc::new(RefCell::new(game.get()));

        View::vstack()
            .spacing(1)
            .child(View::styled_text("Game of Life").bold().build())
            .child(
                View::hstack()
                    .spacing(1)
                    .child(View::styled_text(format!("Gen: {}", generation.get())).dim().build())
                    .child(View::styled_text(format!("Alive: {}", game.get().alive_count())).dim().build())
                    .child(if playing.get() {
                        View::styled_text("PLAYING").color(Color::Green).bold().build()
                    } else {
                        View::styled_text("PAUSED").color(Color::Yellow).build()
                    })
                    .build(),
            )
            .child(View::custom(widget))
            .child(
                View::hstack()
                    .spacing(1)
                    .child(
                        View::button()
                            .label("[ Step ]")
                            .on_press(with!(game, generation => move || {
                                game.update(|g| g.step());
                                generation.update(|n| *n += 1);
                            }))
                            .build(),
                    )
                    .child(
                        View::button()
                            .label(if playing.get() { "[ Pause ]" } else { "[ Play ]" })
                            .on_press(with!(playing => move || playing.update(|p| *p = !*p)))
                            .build(),
                    )
                    .child(
                        View::button()
                            .label("[ Randomize ]")
                            .on_press(with!(game, generation => move || {
                                game.update(|g| g.randomize());
                                generation.set(0);
                            }))
                            .build(),
                    )
                    .child(
                        View::button()
                            .label("[ Clear ]")
                            .on_press(with!(game, generation => move || {
                                game.update(|g| g.clear());
                                generation.set(0);
                            }))
                            .build(),
                    )
                    .build(),
            )
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 38: Custom Widget")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Conway's Game of Life"))
                            .child(View::text("• Custom Widget renders the grid"))
                            .child(View::text("• interval! drives auto-play"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• impl Widget for YourStruct"))
                            .child(View::text("• render(area, buf) draws cells"))
                            .child(View::text("• height_hint / width_hint for sizing"))
                            .child(View::text("• View::custom(Rc<RefCell<W>>)"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Press Play to auto-step"))
                            .child(View::text("• Step manually one at a time"))
                            .child(View::text("• Randomize for a new pattern"))
                            .child(View::text("• Clear then Step to watch"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("-> 39_port: bidirectional comms"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
