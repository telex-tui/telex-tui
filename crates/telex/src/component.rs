use crate::scope::Scope;
use crate::View;

/// A component that can render itself to a View.
pub trait Component {
    /// Render this component to a View tree.
    fn render(&self, cx: Scope) -> View;
}

/// Blanket implementation for closures that take Scope and return View.
/// This enables: rte::run(|cx| view! { ... })
impl<F> Component for F
where
    F: Fn(Scope) -> View,
{
    fn render(&self, cx: Scope) -> View {
        self(cx)
    }
}
