# Custom Widgets

Escape hatch for user-defined character-cell rendering via the `Widget` trait.

```rust
use telex::widget::Widget;
use telex::buffer::{Buffer, Rect};

struct Sparkline {
    data: Vec<u8>,
}

impl Widget for Sparkline {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        for (i, &val) in self.data.iter().enumerate() {
            if i as u16 >= area.width { break; }
            let idx = (val as usize * 7) / 100;
            buf.set(area.x + i as u16, area.y, bars[idx],
                crossterm::style::Color::Cyan, crossterm::style::Color::Reset);
        }
    }
}

View::custom(Rc::new(RefCell::new(sparkline)))
```

## The Widget trait

```rust
pub trait Widget {
    /// Draw into the buffer within the given area
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Whether this widget can receive keyboard focus (default: false)
    fn focusable(&self) -> bool { false }

    /// Preferred height given available width (default: None)
    fn height_hint(&self, _width: u16) -> Option<u16> { None }

    /// Preferred width (default: None)
    fn width_hint(&self) -> Option<u16> { None }
}
```

### render

The only required method. Called each frame with:
- **`area`** — A `Rect { x, y, width, height }` defining the available space
- **`buf`** — A `Buffer` to write characters into

Use `buf.set(x, y, char, fg, bg)` to place characters. Stay within the area bounds.

### focusable

Return `true` if the widget should participate in tab navigation.

### height_hint / width_hint

Return `Some(n)` to suggest a preferred size. The layout engine uses these as hints but may allocate different dimensions.

## Embedding in your UI

Wrap your widget in `Rc<RefCell<_>>` and pass to `View::custom()`:

```rust
use std::rc::Rc;
use std::cell::RefCell;

let widget = Sparkline { data: vec![10, 50, 80, 30, 60] };
View::custom(Rc::new(RefCell::new(widget)))
```

Custom widgets compose with all other views — put them in stacks, boxes, tabs, or anywhere a `View` is accepted.

## Example: spectrum visualizer

```rust
struct Spectrum {
    levels: Vec<f64>,
}

impl Widget for Spectrum {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        for (i, &level) in self.levels.iter().enumerate() {
            if i as u16 >= area.width { break; }

            let height = (level * area.height as f64) as u16;
            for row in 0..area.height {
                let y = area.y + area.height - 1 - row;
                let ch = if row < height { '█' } else { ' ' };
                buf.set(area.x + i as u16, y, ch,
                    crossterm::style::Color::Green,
                    crossterm::style::Color::Reset);
            }
        }
    }

    fn height_hint(&self, _width: u16) -> Option<u16> {
        Some(10)
    }

    fn width_hint(&self) -> Option<u16> {
        Some(self.levels.len() as u16)
    }
}
```

## Tips

**Stay in bounds** — Only write to cells within `area.x..area.x+area.width` and `area.y..area.y+area.height`. Writing outside may corrupt other widgets.

**RefCell for mutation** — Since widgets are behind `Rc<RefCell<_>>`, you can mutate widget state between renders.

**Combine with state** — Create the widget from state values each render, or store the widget in state and update it.

**Performance** — The widget's `render` is called every frame. Keep it fast — no I/O, no allocations if possible.
