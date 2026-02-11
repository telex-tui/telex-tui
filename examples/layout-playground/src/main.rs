//! Interactive layout playground for learning the telex layout system.

use telex::prelude::*;
use telex::Color;

fn main() {
    telex::run_with_theme(
        |cx: Scope| {
            // Layout mode: 0 = HStack, 1 = VStack
            let mode = state!(cx, || 0usize);

            // Flex values for 3 boxes
            let flex_a = state!(cx, || 1u16);
            let flex_b = state!(cx, || 2u16);
            let flex_c = state!(cx, || 1u16);

            // Spacing
            let spacing = state!(cx, || 1u16);

            // Which control is selected (0=mode, 1=A, 2=B, 3=C, 4=spacing)
            let selected = state!(cx, || 1usize);

            // Key handlers
            let sel = selected.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::Tab), move || {
                sel.update(|s| *s = (*s + 1) % 5);
            });

            let sel = selected.clone();
            cx.use_command(
                KeyBinding::new(telex::KeyCode::BackTab, telex::KeyModifiers::SHIFT),
                move || {
                    sel.update(|s| *s = if *s == 0 { 4 } else { *s - 1 });
                },
            );

            // Up/Down to change values
            let mode_up = mode.clone();
            let flex_a_up = flex_a.clone();
            let flex_b_up = flex_b.clone();
            let flex_c_up = flex_c.clone();
            let spacing_up = spacing.clone();
            let sel_up = selected.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::Up), move || {
                match sel_up.get() {
                    0 => mode_up.update(|m| *m = (*m + 1) % 2),
                    1 => flex_a_up.update(|f| *f = (*f + 1).min(10)),
                    2 => flex_b_up.update(|f| *f = (*f + 1).min(10)),
                    3 => flex_c_up.update(|f| *f = (*f + 1).min(10)),
                    4 => spacing_up.update(|s| *s = (*s + 1).min(5)),
                    _ => {}
                }
            });

            let mode_down = mode.clone();
            let flex_a_down = flex_a.clone();
            let flex_b_down = flex_b.clone();
            let flex_c_down = flex_c.clone();
            let spacing_down = spacing.clone();
            let sel_down = selected.clone();
            cx.use_command(
                KeyBinding::key(telex::KeyCode::Down),
                move || match sel_down.get() {
                    0 => mode_down.update(|m| *m = (*m + 1) % 2),
                    1 => flex_a_down.update(|f| *f = f.saturating_sub(1)),
                    2 => flex_b_down.update(|f| *f = f.saturating_sub(1)),
                    3 => flex_c_down.update(|f| *f = f.saturating_sub(1)),
                    4 => spacing_down.update(|s| *s = s.saturating_sub(1)),
                    _ => {}
                },
            );

            // Build the demo layout
            let box_a = View::boxed()
                .border(true)
                .flex(flex_a.get())
                .child(
                    View::vstack()
                        .child(View::styled_text("A").bold().color(Color::Cyan).build())
                        .child(View::text(format!("flex={}", flex_a.get())))
                        .build(),
                )
                .build();

            let box_b = View::boxed()
                .border(true)
                .flex(flex_b.get())
                .child(
                    View::vstack()
                        .child(View::styled_text("B").bold().color(Color::Yellow).build())
                        .child(View::text(format!("flex={}", flex_b.get())))
                        .build(),
                )
                .build();

            let box_c = View::boxed()
                .border(true)
                .flex(flex_c.get())
                .child(
                    View::vstack()
                        .child(View::styled_text("C").bold().color(Color::Magenta).build())
                        .child(View::text(format!("flex={}", flex_c.get())))
                        .build(),
                )
                .build();

            let demo_layout = if mode.get() == 0 {
                View::hstack()
                    .spacing(spacing.get())
                    .child(box_a)
                    .child(box_b)
                    .child(box_c)
                    .build()
            } else {
                View::vstack()
                    .spacing(spacing.get())
                    .child(box_a)
                    .child(box_b)
                    .child(box_c)
                    .build()
            };

            // Calculate percentages for display
            let total_flex = flex_a.get() + flex_b.get() + flex_c.get();
            let pct_a = if total_flex > 0 {
                (flex_a.get() as f32 / total_flex as f32 * 100.0) as u16
            } else {
                0
            };
            let pct_b = if total_flex > 0 {
                (flex_b.get() as f32 / total_flex as f32 * 100.0) as u16
            } else {
                0
            };
            let pct_c = if total_flex > 0 {
                (flex_c.get() as f32 / total_flex as f32 * 100.0) as u16
            } else {
                0
            };

            // Control panel
            let sel_val = selected.get();
            let mode_str = if mode.get() == 0 { "HStack" } else { "VStack" };

            let control = |label: &str, value: String, is_selected: bool| -> View {
                let style = if is_selected {
                    View::styled_text(format!(" [{}]: {} ", label, value))
                        .bold()
                        .color(Color::Black)
                        .bg(Color::White)
                        .build()
                } else {
                    View::text(format!("  {}: {} ", label, value))
                };
                style
            };

            View::vstack()
                .spacing(0)
                // Header
                .child(
                    View::boxed()
                        .border(true)
                        .child(
                            View::hstack()
                                .child(
                                    View::styled_text("Layout Playground")
                                        .bold()
                                        .color(Color::Green)
                                        .build(),
                                )
                                .child(View::text(
                                    " - Tab to select, Up/Down to change, Ctrl+Q to quit",
                                ))
                                .build(),
                        )
                        .build(),
                )
                // Demo area
                .child(
                    View::boxed()
                        .border(true)
                        .flex(1)
                        .child(demo_layout)
                        .build(),
                )
                // Percentage display
                .child(
                    View::boxed()
                        .border(true)
                        .child(
                            View::hstack()
                                .child(View::text(format!(
                                    "Distribution: A={}% B={}% C={}%",
                                    pct_a, pct_b, pct_c
                                )))
                                .child(View::text(format!("  |  Total flex: {}", total_flex)))
                                .build(),
                        )
                        .build(),
                )
                // Controls
                .child(
                    View::boxed()
                        .border(true)
                        .child(
                            View::hstack()
                                .spacing(2)
                                .child(control("Mode", mode_str.to_string(), sel_val == 0))
                                .child(control("A flex", flex_a.get().to_string(), sel_val == 1))
                                .child(control("B flex", flex_b.get().to_string(), sel_val == 2))
                                .child(control("C flex", flex_c.get().to_string(), sel_val == 3))
                                .child(control("Spacing", spacing.get().to_string(), sel_val == 4))
                                .build(),
                        )
                        .build(),
                )
                .build()
        },
        telex::theme::Theme::nord(),
    )
    .unwrap();
}
