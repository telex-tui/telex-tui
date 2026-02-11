//! Procedural macros for Telex.
//!
//! - `state!` — creates order-independent state (no hook ordering rules)
//! - `effect!` — creates order-independent effects with dependencies
//! - `effect_once!` — creates order-independent effects that run once
//! - `with!` — clones state handles into closures
//! - `view!` — JSX-like syntax for building UI trees

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::sync::atomic::{AtomicU64, Ordering};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, LitStr, Result, Token,
};

/// The view! macro for building UI trees with JSX-like syntax.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let node = parse_macro_input!(input as ViewNode);
    let expanded = node.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the with! macro: `ident1, ident2 => expr`
struct WithInput {
    idents: Vec<Ident>,
    expr: Expr,
}

impl Parse for WithInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse comma-separated identifiers
        let idents: Punctuated<Ident, Token![,]> = Punctuated::parse_separated_nonempty(input)?;
        let idents: Vec<Ident> = idents.into_iter().collect();

        // Parse the => separator
        input.parse::<Token![=>]>()?;

        // Parse the expression (typically a closure)
        let expr: Expr = input.parse()?;

        Ok(WithInput { idents, expr })
    }
}

impl WithInput {
    fn to_tokens(&self) -> TokenStream2 {
        let clone_statements: Vec<TokenStream2> = self
            .idents
            .iter()
            .map(|ident| quote! { let #ident = #ident.clone(); })
            .collect();

        let expr = &self.expr;

        quote! {
            {
                #(#clone_statements)*
                #expr
            }
        }
    }
}

/// Global counter for generating unique type names.
static STATE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Input for the state! macro: `cx, || init_expr`
struct StateInput {
    scope: Expr,
    init: Expr,
}

impl Parse for StateInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the scope expression (usually just `cx`)
        let scope: Expr = input.parse()?;

        // Parse the comma separator
        input.parse::<Token![,]>()?;

        // Parse the initializer expression (usually a closure)
        let init: Expr = input.parse()?;

        Ok(StateInput { scope, init })
    }
}

impl StateInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let init = &self.init;

        // Generate a unique type name using an atomic counter.
        // This ensures each macro invocation gets a distinct type.
        let counter = STATE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__State_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_state_keyed::<#key_type, _>(#init)
            }
        }
    }
}

/// The state! macro for creating order-independent state.
///
/// This is the recommended way to create state in Telex. Unlike traditional
/// hooks, state created with this macro can be used conditionally or in any
/// order without causing panics.
///
/// Each macro invocation creates a unique anonymous type as the key,
/// ensuring each call site gets its own independent state.
///
/// # Examples
///
/// Basic usage:
/// ```ignore
/// let count = state!(cx, || 0);
/// ```
///
/// Safe in conditionals:
/// ```ignore
/// if show_counter {
///     let count = state!(cx, || 0);  // This is safe!
/// }
/// ```
///
/// Multiple independent states:
/// ```ignore
/// let name = state!(cx, || String::new());
/// let count = state!(cx, || 0);
/// let visible = state!(cx, || true);
/// ```
#[proc_macro]
pub fn state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as StateInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Global counter for generating unique effect type names.
static EFFECT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Input for the effect! macro: `cx, deps, |&d| effect_body`
struct EffectInput {
    scope: Expr,
    deps: Expr,
    effect_fn: Expr,
}

impl Parse for EffectInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the scope expression (usually just `cx`)
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse the dependencies expression
        let deps: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse the effect closure
        let effect_fn: Expr = input.parse()?;

        Ok(EffectInput {
            scope,
            deps,
            effect_fn,
        })
    }
}

impl EffectInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let deps = &self.deps;
        let effect_fn = &self.effect_fn;

        let counter = EFFECT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Effect_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_effect_keyed::<#key_type, _, _, _>(#deps, #effect_fn)
            }
        }
    }
}

/// The effect! macro for creating order-independent effects with dependencies.
///
/// This is the recommended way to create effects in Telex. Unlike traditional
/// hooks, effects created with this macro can be used conditionally or in any
/// order without causing issues.
///
/// Each macro invocation creates a unique anonymous type as the key,
/// ensuring each call site gets its own independent effect.
///
/// # Examples
///
/// Basic usage - runs when count changes:
/// ```ignore
/// effect!(cx, count.get(), |&c| {
///     println!("count changed to {}", c);
///     || {}  // cleanup function
/// });
/// ```
///
/// Safe in conditionals:
/// ```ignore
/// if show_logger {
///     effect!(cx, value.get(), |&v| {
///         println!("value: {}", v);
///         || {}
///     });
/// }
/// ```
///
/// Multiple dependencies via tuple:
/// ```ignore
/// effect!(cx, (a.get(), b.get()), |&(a, b)| {
///     println!("a={}, b={}", a, b);
///     || {}
/// });
/// ```
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EffectInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the effect_once! macro: `cx, || effect_body`
struct EffectOnceInput {
    scope: Expr,
    effect_fn: Expr,
}

impl Parse for EffectOnceInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the scope expression (usually just `cx`)
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse the effect closure
        let effect_fn: Expr = input.parse()?;

        Ok(EffectOnceInput { scope, effect_fn })
    }
}

impl EffectOnceInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let effect_fn = &self.effect_fn;

        let counter = EFFECT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Effect_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_effect_once_keyed::<#key_type, _, _>(#effect_fn)
            }
        }
    }
}

/// The effect_once! macro for creating order-independent effects that run once.
///
/// This is the recommended way to run one-time initialization effects in Telex.
/// Unlike traditional hooks, effects created with this macro can be used
/// conditionally or in any order.
///
/// Each macro invocation creates a unique anonymous type as the key,
/// ensuring each call site gets its own independent effect.
///
/// # Examples
///
/// Basic usage - runs once on first render:
/// ```ignore
/// effect_once!(cx, || {
///     println!("App initialized");
///     || {
///         println!("App cleanup");
///     }
/// });
/// ```
///
/// Safe in conditionals:
/// ```ignore
/// if feature_enabled {
///     effect_once!(cx, || {
///         setup_feature();
///         || cleanup_feature()
///     });
/// }
/// ```
#[proc_macro]
pub fn effect_once(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EffectOnceInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Global counter for generating unique async/stream/terminal type names.
static HOOK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Input for the async_data! macro: `cx, || { ... }`
struct AsyncDataInput {
    scope: Expr,
    func: Expr,
}

impl Parse for AsyncDataInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let func: Expr = input.parse()?;
        Ok(AsyncDataInput { scope, func })
    }
}

impl AsyncDataInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let func = &self.func;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Async_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_async_keyed::<#key_type, _, _>(#func)
            }
        }
    }
}

/// The async_data! macro for creating order-independent async data loading.
///
/// # Examples
///
/// ```ignore
/// let data = async_data!(cx, || {
///     Ok(fetch_data())
/// });
/// ```
#[proc_macro]
pub fn async_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AsyncDataInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the stream! macro: `cx, || { ... }`
struct StreamInput {
    scope: Expr,
    func: Expr,
}

impl Parse for StreamInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let func: Expr = input.parse()?;
        Ok(StreamInput { scope, func })
    }
}

impl StreamInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let func = &self.func;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Stream_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_stream_keyed::<#key_type, _, _, _>(#func)
            }
        }
    }
}

/// The stream! macro for creating order-independent streams.
///
/// # Examples
///
/// ```ignore
/// let elapsed = stream!(cx, || {
///     (0..).inspect(|_| std::thread::sleep(Duration::from_secs(1)))
/// });
/// ```
#[proc_macro]
pub fn stream(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as StreamInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the text_stream! macro: `cx, || { ... }`
struct TextStreamInput {
    scope: Expr,
    func: Expr,
}

impl Parse for TextStreamInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let func: Expr = input.parse()?;
        Ok(TextStreamInput { scope, func })
    }
}

impl TextStreamInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let func = &self.func;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__TextStream_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_text_stream_keyed::<#key_type, _, _>(#func)
            }
        }
    }
}

/// The text_stream! macro for creating order-independent text streams.
///
/// # Examples
///
/// ```ignore
/// let logs = text_stream!(cx, || {
///     generate_log_entries()
/// });
/// ```
#[proc_macro]
pub fn text_stream(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TextStreamInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the text_stream_with_restart! macro: `cx, restart, || { ... }`
struct TextStreamWithRestartInput {
    scope: Expr,
    restart: Expr,
    func: Expr,
}

impl Parse for TextStreamWithRestartInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let restart: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let func: Expr = input.parse()?;
        Ok(TextStreamWithRestartInput { scope, restart, func })
    }
}

impl TextStreamWithRestartInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let restart = &self.restart;
        let func = &self.func;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__TextStreamRestart_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_text_stream_with_restart_keyed::<#key_type, _, _>(#restart, #func)
            }
        }
    }
}

/// The text_stream_with_restart! macro for creating restartable text streams.
///
/// # Examples
///
/// ```ignore
/// let stream = text_stream_with_restart!(cx, needs_restart, move || {
///     stream_response()
/// });
/// ```
#[proc_macro]
pub fn text_stream_with_restart(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TextStreamWithRestartInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the terminal! macro: `cx`
struct TerminalInput {
    scope: Expr,
}

impl Parse for TerminalInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        Ok(TerminalInput { scope })
    }
}

impl TerminalInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Terminal_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_terminal_keyed::<#key_type>()
            }
        }
    }
}

/// The terminal! macro for creating order-independent terminal handles.
///
/// # Examples
///
/// ```ignore
/// let terminal = terminal!(cx);
/// ```
#[proc_macro]
pub fn terminal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TerminalInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the reducer! macro: `cx, initial, |state, action| { ... }`
struct ReducerInput {
    scope: Expr,
    initial: Expr,
    reducer_fn: Expr,
}

impl Parse for ReducerInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let initial: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let reducer_fn: Expr = input.parse()?;
        Ok(ReducerInput { scope, initial, reducer_fn })
    }
}

impl ReducerInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let initial = &self.initial;
        let reducer_fn = &self.reducer_fn;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Reducer_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_reducer_keyed::<#key_type, _, _>(#initial, #reducer_fn)
            }
        }
    }
}

/// The reducer! macro for creating order-independent state with dispatch.
///
/// Returns `(state, dispatch)` where `dispatch` sends actions through
/// a reducer function to produce new state.
///
/// # Examples
///
/// ```ignore
/// let (state, dispatch) = reducer!(cx, AppState::Idle, |state, action| {
///     match (state, action) {
///         (_, Action::Reset) => AppState::Idle,
///         (s, _) => s,
///     }
/// });
///
/// // Dispatch an action
/// dispatch(Action::Reset);
/// ```
#[proc_macro]
pub fn reducer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ReducerInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the channel! macro: `cx, Type`
struct ChannelInput {
    scope: Expr,
    ty: syn::Type,
}

impl Parse for ChannelInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty: syn::Type = input.parse()?;
        Ok(ChannelInput { scope, ty })
    }
}

impl ChannelInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let ty = &self.ty;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Channel_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_channel_keyed::<#key_type, #ty>()
            }
        }
    }
}

/// The channel! macro for creating typed inbound channels.
///
/// # Examples
///
/// ```ignore
/// let ch = channel!(cx, String);
/// let tx = ch.tx();
/// // Send tx to an external thread via effect_once!
/// for msg in ch.get() { /* ... */ }
/// ```
#[proc_macro]
pub fn channel(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ChannelInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the port! macro: `cx, InType, OutType`
struct PortInput {
    scope: Expr,
    in_ty: syn::Type,
    out_ty: syn::Type,
}

impl Parse for PortInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let in_ty: syn::Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let out_ty: syn::Type = input.parse()?;
        Ok(PortInput { scope, in_ty, out_ty })
    }
}

impl PortInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let in_ty = &self.in_ty;
        let out_ty = &self.out_ty;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Port_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_port_keyed::<#key_type, #in_ty, #out_ty>()
            }
        }
    }
}

/// The port! macro for creating bidirectional ports.
///
/// # Examples
///
/// ```ignore
/// let midi = port!(cx, MidiIn, MidiOut);
/// let inbound_tx = midi.rx.tx();   // external sends MidiIn here
/// let outbound_tx = midi.tx();     // component sends MidiOut here
/// for msg in midi.rx.get() { /* ... */ }
/// ```
#[proc_macro]
pub fn port(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PortInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// Input for the interval! macro: `cx, duration, || { ... }`
struct IntervalInput {
    scope: Expr,
    duration: Expr,
    callback: Expr,
}

impl Parse for IntervalInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let scope: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let duration: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let callback: Expr = input.parse()?;
        Ok(IntervalInput { scope, duration, callback })
    }
}

impl IntervalInput {
    fn to_tokens(&self) -> TokenStream2 {
        let scope = &self.scope;
        let duration = &self.duration;
        let callback = &self.callback;
        let counter = HOOK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let key_type = format_ident!("__Interval_{}", counter);

        quote! {
            {
                struct #key_type;
                #scope.use_interval_keyed::<#key_type>(#duration, #callback)
            }
        }
    }
}

/// The interval! macro for creating periodic timers.
///
/// The callback runs on the main thread each frame that the timer fires.
///
/// # Examples
///
/// ```ignore
/// let count = state!(cx, || 0u64);
/// let c = count.clone();
/// interval!(cx, Duration::from_secs(1), move || {
///     c.update(|n| *n += 1);
/// });
/// ```
#[proc_macro]
pub fn interval(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IntervalInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// The with! macro for cloning state handles into closures.
///
/// State<T> is a handle (like a smart pointer), not the data itself.
/// When you need to use state in a closure, you must clone the handle
/// so the closure owns its own copy. This macro makes that pattern concise.
///
/// # Examples
///
/// Single state:
/// ```ignore
/// let count = state!(cx, || 0);
/// let increment = with!(count => move || count.update(|n| *n += 1));
/// ```
///
/// Multiple states:
/// ```ignore
/// let count = state!(cx, || 0);
/// let name = state!(cx, || String::new());
///
/// let handler = with!(count, name => move || {
///     count.update(|n| *n += 1);
///     name.set("updated".to_string());
/// });
/// ```
///
/// The above expands to:
/// ```ignore
/// let handler = {
///     let count = count.clone();
///     let name = name.clone();
///     move || {
///         count.update(|n| *n += 1);
///         name.set("updated".to_string());
///     }
/// };
/// ```
#[proc_macro]
pub fn with(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as WithInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

/// A node in the view tree (during parsing).
enum ViewNode {
    /// An element like <Text>...</Text>
    Element(ElementNode),
    /// A string literal "Hello"
    Text(String),
    /// An expression in braces {expr}
    Expr(Expr),
}

/// A prop like on_press={...} or selected={...}
struct Prop {
    name: Ident,
    value: Expr,
}

struct ElementNode {
    tag: String,
    props: Vec<Prop>,
    children: Vec<ViewNode>,
}

impl Parse for ViewNode {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![<]) {
            // Parse element: <Tag prop={val}>...</Tag>
            input.parse::<Token![<]>()?;
            let tag: Ident = input.parse()?;

            // Parse props
            let mut props = Vec::new();
            while !input.peek(Token![>]) && !input.peek(Token![/]) {
                let name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let content;
                braced!(content in input);
                let value: Expr = content.parse()?;
                props.push(Prop { name, value });
            }

            // Check for self-closing tag: <Tag />
            if input.peek(Token![/]) {
                input.parse::<Token![/]>()?;
                input.parse::<Token![>]>()?;
                return Ok(ViewNode::Element(ElementNode {
                    tag: tag.to_string(),
                    props,
                    children: Vec::new(),
                }));
            }

            input.parse::<Token![>]>()?;

            let mut children = Vec::new();

            // Parse children until we hit the closing tag
            while !(input.peek(Token![<]) && input.peek2(Token![/])) {
                if input.is_empty() {
                    return Err(syn::Error::new(
                        tag.span(),
                        format!("Unclosed tag: <{}>", tag),
                    ));
                }
                children.push(input.parse()?);
            }

            // Parse closing tag: </Tag>
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            let close_tag: Ident = input.parse()?;
            input.parse::<Token![>]>()?;

            if tag != close_tag {
                return Err(syn::Error::new(
                    close_tag.span(),
                    format!(
                        "Mismatched tags: expected </{}>, found </{}>",
                        tag, close_tag
                    ),
                ));
            }

            Ok(ViewNode::Element(ElementNode {
                tag: tag.to_string(),
                props,
                children,
            }))
        } else if input.peek(LitStr) {
            // Parse string literal: "Hello"
            let lit: LitStr = input.parse()?;
            Ok(ViewNode::Text(lit.value()))
        } else if input.peek(syn::token::Brace) {
            // Parse expression: {expr}
            let content;
            braced!(content in input);
            let expr: Expr = content.parse()?;
            Ok(ViewNode::Expr(expr))
        } else {
            Err(input.error("Expected <Element>, \"string literal\", or {expression}"))
        }
    }
}

impl ViewNode {
    fn to_tokens(&self) -> TokenStream2 {
        match self {
            ViewNode::Text(s) => {
                quote! { telex::View::text(#s) }
            }
            ViewNode::Expr(expr) => {
                // Convert expression to string for text
                quote! { telex::View::text(format!("{}", #expr)) }
            }
            ViewNode::Element(elem) => elem.to_tokens(),
        }
    }
}

impl ElementNode {
    fn to_tokens(&self) -> TokenStream2 {
        match self.tag.as_str() {
            "Text" => {
                // <Text>"content"</Text> or <Text>{expr}</Text>
                if let Some(child) = self.children.first() {
                    match child {
                        ViewNode::Text(content) => quote! { telex::View::text(#content) },
                        ViewNode::Expr(expr) => quote! { telex::View::text(format!("{}", #expr)) },
                        _ => quote! { telex::View::text("") },
                    }
                } else {
                    quote! { telex::View::text("") }
                }
            }
            "VStack" => {
                let mut builder_calls = Vec::new();

                // Handle props (spacing)
                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                // Handle children
                for child in &self.children {
                    let tokens = child.to_tokens();
                    builder_calls.push(quote! { .child(#tokens) });
                }

                quote! { telex::View::vstack()#(#builder_calls)*.build() }
            }
            "HStack" => {
                let mut builder_calls = Vec::new();

                // Handle props (spacing)
                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                // Handle children
                for child in &self.children {
                    let tokens = child.to_tokens();
                    builder_calls.push(quote! { .child(#tokens) });
                }

                quote! { telex::View::hstack()#(#builder_calls)*.build() }
            }
            "Box" => {
                let mut builder_calls = Vec::new();

                // Handle props (border, padding, flex)
                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                // Handle single child
                if let Some(child) = self.children.first() {
                    let tokens = child.to_tokens();
                    builder_calls.push(quote! { .child(#tokens) });
                }

                quote! { telex::View::boxed()#(#builder_calls)*.build() }
            }
            "Spacer" => {
                // Spacer with optional flex prop
                if let Some(prop) = self.props.iter().find(|p| p.name == "flex") {
                    let value = &prop.value;
                    quote! { telex::View::spacer_flex(#value) }
                } else {
                    quote! { telex::View::spacer() }
                }
            }
            "Button" => {
                // Parse props and children for Button
                let mut builder_calls = Vec::new();

                // Handle props
                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                // Handle label from children
                if let Some(child) = self.children.first() {
                    match child {
                        ViewNode::Text(label) => {
                            builder_calls.push(quote! { .label(#label) });
                        }
                        ViewNode::Expr(expr) => {
                            builder_calls.push(quote! { .label(format!("{}", #expr)) });
                        }
                        _ => {}
                    }
                }

                quote! { telex::View::button()#(#builder_calls)*.build() }
            }
            "List" => {
                // Parse props for List: items, selected, on_select
                let mut builder_calls = Vec::new();

                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                quote! { telex::View::list()#(#builder_calls)*.build() }
            }
            "TextInput" => {
                // Parse props for TextInput: value, placeholder, on_change
                let mut builder_calls = Vec::new();

                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                quote! { telex::View::text_input()#(#builder_calls)*.build() }
            }
            "Checkbox" => {
                // Parse props and children for Checkbox: checked, on_toggle
                let mut builder_calls = Vec::new();

                // Handle props
                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                // Handle label from children
                if let Some(child) = self.children.first() {
                    match child {
                        ViewNode::Text(label) => {
                            builder_calls.push(quote! { .label(#label) });
                        }
                        ViewNode::Expr(expr) => {
                            builder_calls.push(quote! { .label(format!("{}", #expr)) });
                        }
                        _ => {}
                    }
                }

                quote! { telex::View::checkbox()#(#builder_calls)*.build() }
            }
            "TextArea" => {
                // Parse props for TextArea: value, placeholder, rows, cursor_line, cursor_col, on_change
                let mut builder_calls = Vec::new();

                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                quote! { telex::View::text_area()#(#builder_calls)*.build() }
            }
            "Modal" => {
                // Parse props for Modal: visible, title, width, height, on_dismiss
                let mut builder_calls = Vec::new();

                for prop in &self.props {
                    let name = &prop.name;
                    let value = &prop.value;
                    builder_calls.push(quote! { .#name(#value) });
                }

                // Handle single child
                if let Some(child) = self.children.first() {
                    let tokens = child.to_tokens();
                    builder_calls.push(quote! { .child(#tokens) });
                }

                quote! { telex::View::modal()#(#builder_calls)*.build() }
            }
            "StyledText" => {
                // Parse props for styled text: bold, italic, underline, dim, color, bg
                let mut content = quote! { "" };
                let mut bold_val = quote! { false };
                let mut italic_val = quote! { false };
                let mut underline_val = quote! { false };
                let mut dim_val = quote! { false };
                let mut color_call = quote! {};
                let mut bg_call = quote! {};

                // Handle props
                for prop in &self.props {
                    let name_str = prop.name.to_string();
                    let value = &prop.value;

                    match name_str.as_str() {
                        "bold" => bold_val = quote! { #value },
                        "italic" => italic_val = quote! { #value },
                        "underline" => underline_val = quote! { #value },
                        "dim" => dim_val = quote! { #value },
                        "color" => color_call = quote! { .color(#value) },
                        "bg" => bg_call = quote! { .bg(#value) },
                        _ => {}
                    }
                }

                // Handle text content from children
                if let Some(child) = self.children.first() {
                    match child {
                        ViewNode::Text(text) => {
                            content = quote! { #text };
                        }
                        ViewNode::Expr(expr) => {
                            content = quote! { format!("{}", #expr) };
                        }
                        _ => {}
                    }
                }

                // Generate conditional builder chain
                quote! {
                    {
                        let __builder = telex::View::styled_text(#content);
                        let __builder = if #bold_val { __builder.bold() } else { __builder };
                        let __builder = if #italic_val { __builder.italic() } else { __builder };
                        let __builder = if #underline_val { __builder.underline() } else { __builder };
                        let __builder = if #dim_val { __builder.dim() } else { __builder };
                        __builder #color_call #bg_call .build()
                    }
                }
            }
            unknown => {
                // Provide helpful error with suggestions
                let known_elements = [
                    "Text",
                    "StyledText",
                    "VStack",
                    "HStack",
                    "Box",
                    "Spacer",
                    "Button",
                    "List",
                    "TextInput",
                    "TextArea",
                    "Checkbox",
                    "Modal",
                ];

                // Find similar element names (simple edit distance check)
                let suggestion = known_elements
                    .iter()
                    .find(|&e| {
                        let e_lower = e.to_lowercase();
                        let u_lower = unknown.to_lowercase();
                        e_lower.starts_with(&u_lower[..1.min(u_lower.len())])
                            || u_lower.starts_with(&e_lower[..1.min(e_lower.len())])
                            || e_lower.contains(&u_lower)
                            || u_lower.contains(&e_lower)
                    });

                let msg = if let Some(suggested) = suggestion {
                    format!(
                        "Unknown element: <{}>. Did you mean <{}>?\n\nAvailable elements: {}",
                        unknown,
                        suggested,
                        known_elements.join(", ")
                    )
                } else {
                    format!(
                        "Unknown element: <{}>.\n\nAvailable elements: {}",
                        unknown,
                        known_elements.join(", ")
                    )
                };
                quote! { compile_error!(#msg) }
            }
        }
    }
}
