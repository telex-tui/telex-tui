# Layouts

Telex uses a **flexbox-inspired layout model**. If you've used CSS flexbox or SwiftUI stacks, you'll feel at home.

## The mental model

Layout in Telex works like this:

1. **Stacks arrange children** along one axis - `VStack` goes top-to-bottom, `HStack` goes left-to-right
2. **Children have intrinsic sizes** - a button is 1 line tall, text wraps to fit width
3. **Flex distributes extra space** - children with `flex(1)` grow to fill remaining room
4. **Constraints set boundaries** - `min_height`, `max_height` limit how much a child can grow or shrink

The algorithm is simple: measure fixed children first, then divide leftover space among flexible children proportionally.

```
┌─────────────────────────────┐
│ Header (intrinsic: 1 line)  │  ← fixed
├─────────────────────────────┤
│                             │
│ Content (flex: 1)           │  ← grows to fill
│                             │
├─────────────────────────────┤
│ Footer (intrinsic: 1 line)  │  ← fixed
└─────────────────────────────┘
```

**No absolute positioning.** No z-index. No CSS grid. Just stacks, flex, and splits. This keeps the model simple and predictable.

---

## Vertical stack

```rust
View::vstack()
    .spacing(1)  // gap between children
    .child(View::text("First"))
    .child(View::text("Second"))
    .child(View::text("Third"))
    .build()
```

## Horizontal stack

```rust
View::hstack()
    .spacing(2)
    .child(View::text("Left"))
    .child(View::text("Right"))
    .build()
```

## Flex

Use `.flex(1)` to make an element fill available space:

```rust
View::vstack()
    .child(View::text("Header"))
    .child(
        View::boxed()
            .flex(1)  // fills remaining space
            .child(View::text("Content"))
            .build()
    )
    .child(View::text("Footer"))
    .build()
```

## Spacer and gap

```rust
View::spacer()  // fills available space
View::gap(2)    // fixed 2-line gap
```

## Split panes

```rust
View::split()
    .horizontal()
    .ratio(0.3)  // 30% left, 70% right
    .first(left_content)
    .second(right_content)
    .build()
```

Run with: `cargo run -p telex-tui --example 13_split_panes`
