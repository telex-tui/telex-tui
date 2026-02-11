// Terminal widget is experimental - see View::terminal() docs for limitations.

use telex::prelude::*;

fn app(cx: Scope) -> View {
    let terminal = terminal!(cx);

    // Spawn bash on first render
    if !terminal.is_started() {
        if let Err(e) = terminal.spawn("bash", &[], 80, 24) {
            eprintln!("Failed to spawn terminal: {}", e);
        }
    }

    View::vstack()
        .child(View::text("Telex Terminal Demo"))
        .child(View::text(
            "Press Ctrl+Shift+[ to escape terminal focus, Tab to navigate",
        ))
        .child(View::terminal().handle(terminal).build())
        .build()
}

fn main() {
    telex::run(app).unwrap();
}
