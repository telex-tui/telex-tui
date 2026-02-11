use std::cell::RefCell;
use std::rc::Rc;

use crate::widget::Widget;

/// Callback type for event handlers (no arguments).
pub type Callback = Rc<dyn Fn()>;

/// Alignment along the main axis (justify).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Justify {
    /// Items at the start (default).
    #[default]
    Start,
    /// Items at the end.
    End,
    /// Items centered.
    Center,
    /// Items spread with space between them.
    SpaceBetween,
    /// Items spread with space around them.
    SpaceAround,
}

/// Alignment along the cross axis (align).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    /// Items at the start.
    Start,
    /// Items at the end.
    End,
    /// Items centered.
    Center,
    /// Items stretch to fill (default).
    #[default]
    Stretch,
}

/// Layout mode for stack containers.
///
/// This enum allows switching between different layout algorithms.
/// Currently only Flex is implemented, but this provides the hook
/// for future layout experiments (e.g., percentage-based layouts).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LayoutMode {
    /// Flex-based layout (default).
    /// Children with flex > 0 share remaining space proportionally.
    /// Children with flex = 0 use their intrinsic/min size.
    #[default]
    Flex,
    // Future: Percent - children specify exact percentages
    // Future: Grid - CSS grid-like layout
}

/// Callback type for selection events (receives selected index).
pub type SelectCallback = Rc<dyn Fn(usize)>;

/// Callback type for text change events (receives new text).
pub type ChangeCallback = Rc<dyn Fn(String)>;

/// Callback type for toggle events (receives new state).
pub type ToggleCallback = Rc<dyn Fn(bool)>;

/// Callback type for cursor position change events (receives line, column).
pub type CursorChangeCallback = Rc<dyn Fn(usize, usize)>;

/// Callback type for cursor position change events in single-line inputs (receives position).
pub type CursorPosCallback = Rc<dyn Fn(usize)>;

/// Path to a node in a tree (indices at each level).
pub type TreePath = Vec<usize>;

/// Callback type for tree selection events (receives path to selected item).
pub type TreeSelectCallback = Rc<dyn Fn(TreePath)>;

/// Callback type for tree activation events (receives path to activated item).
pub type TreeActivateCallback = Rc<dyn Fn(TreePath)>;

/// Callback type for table sort events (receives column index and ascending flag).
pub type SortCallback = Rc<dyn Fn(usize, bool)>;

/// Callback type for table row activation events (receives row index).
pub type RowActivateCallback = Rc<dyn Fn(usize)>;

/// Callback type for command execution events (receives command ID).
pub type CommandCallback = Rc<dyn Fn(&'static str)>;

/// Callback type for canvas drawing (receives mutable draw context).
pub type CanvasDrawCallback = Rc<dyn Fn(&mut crate::canvas::DrawContext)>;

/// Callback type for slider value changes.
pub type SliderCallback = Rc<dyn Fn(f64)>;

/// The core view type - a node in the UI tree.
#[derive(Clone)]
pub enum View {
    /// A text node displaying a string.
    Text(TextNode),
    /// A vertical stack of child views.
    VStack(VStackNode),
    /// A horizontal stack of child views.
    HStack(HStackNode),
    /// A clickable button.
    Button(ButtonNode),
    /// A container with optional border, padding, and flex sizing.
    Box(BoxNode),
    /// Flexible space that expands to fill available space.
    Spacer(SpacerNode),
    /// A selectable list of items.
    List(ListNode),
    /// A single-line text input.
    TextInput(TextInputNode),
    /// A multi-line text area.
    TextArea(TextAreaNode),
    /// A checkbox (toggle).
    Checkbox(CheckboxNode),
    /// A group of radio buttons (mutually exclusive options).
    RadioGroup(RadioGroupNode),
    /// A modal dialog overlay.
    Modal(ModalNode),
    /// A split pane container with two resizable panels.
    Split(SplitNode),
    /// A tabbed interface container.
    Tabs(TabsNode),
    /// A hierarchical tree view.
    Tree(TreeNode),
    /// A data table with columns and rows.
    Table(TableNode),
    /// A progress bar showing completion status.
    ProgressBar(ProgressBarNode),
    /// A status bar displayed at the bottom of the screen.
    StatusBar(StatusBarNode),
    /// A command palette overlay for searching and executing commands.
    CommandPalette(CommandPaletteNode),
    /// A horizontal menu bar with dropdown menus.
    MenuBar(MenuBarNode),
    /// A container for toast notifications.
    ToastContainer(ToastContainerNode),
    /// A form container with validation support.
    Form(FormNode),
    /// A form field with label and error display.
    FormField(FormFieldNode),
    /// A pixel-level canvas using Kitty graphics protocol.
    Canvas(CanvasNode),
    /// An image display using Kitty graphics protocol.
    Image(ImageNode),
    /// An interactive PTY terminal emulator.
    Terminal(TerminalNode),
    /// An error boundary that catches panics in its child view.
    ErrorBoundary(ErrorBoundaryNode),
    /// A user-defined custom widget.
    Custom(CustomNode),
    /// A slider for bounded numeric values.
    Slider(SliderNode),
    /// An empty placeholder.
    Empty,
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            View::Text(n) => f.debug_tuple("Text").field(n).finish(),
            View::VStack(n) => f.debug_tuple("VStack").field(n).finish(),
            View::HStack(n) => f.debug_tuple("HStack").field(n).finish(),
            View::Button(n) => f
                .debug_struct("Button")
                .field("label", &n.label)
                .field("on_press", &"<callback>")
                .finish(),
            View::Box(n) => f.debug_tuple("Box").field(n).finish(),
            View::Spacer(n) => f.debug_tuple("Spacer").field(n).finish(),
            View::List(n) => f
                .debug_struct("List")
                .field("items", &n.items.len())
                .field("selected", &n.selected)
                .finish(),
            View::TextInput(n) => f
                .debug_struct("TextInput")
                .field("value", &n.value)
                .finish(),
            View::TextArea(n) => f
                .debug_struct("TextArea")
                .field("value", &n.value)
                .field("cursor", &(n.cursor_line, n.cursor_col))
                .finish(),
            View::Checkbox(n) => f
                .debug_struct("Checkbox")
                .field("checked", &n.checked)
                .field("label", &n.label)
                .finish(),
            View::RadioGroup(n) => f
                .debug_struct("RadioGroup")
                .field("selected", &n.selected)
                .field("options", &n.options)
                .finish(),
            View::Modal(n) => f
                .debug_struct("Modal")
                .field("visible", &n.visible)
                .field("title", &n.title)
                .finish(),
            View::Split(n) => f
                .debug_struct("Split")
                .field("orientation", &n.orientation)
                .field("ratio", &n.ratio)
                .finish(),
            View::Tabs(n) => f
                .debug_struct("Tabs")
                .field("tabs", &n.tabs)
                .field("active", &n.active)
                .finish(),
            View::Tree(n) => f
                .debug_struct("Tree")
                .field("items", &n.items.len())
                .field("selected", &n.selected)
                .finish(),
            View::Table(n) => f
                .debug_struct("Table")
                .field("columns", &n.columns.len())
                .field("rows", &n.rows.len())
                .field("selected", &n.selected)
                .finish(),
            View::ProgressBar(n) => f
                .debug_struct("ProgressBar")
                .field("value", &n.value)
                .field("label", &n.label)
                .finish(),
            View::StatusBar(n) => f
                .debug_struct("StatusBar")
                .field("left", &n.left)
                .field("center", &n.center)
                .field("right", &n.right)
                .finish(),
            View::CommandPalette(n) => f
                .debug_struct("CommandPalette")
                .field("visible", &n.visible)
                .field("query", &n.query)
                .field("selected", &n.selected)
                .finish(),
            View::MenuBar(n) => f
                .debug_struct("MenuBar")
                .field("menus", &n.menus.len())
                .field("active_menu", &n.active_menu)
                .finish(),
            View::ToastContainer(n) => f
                .debug_struct("ToastContainer")
                .field("toasts", &n.toasts.len())
                .finish(),
            View::Form(n) => f
                .debug_struct("Form")
                .field("children", &n.children.len())
                .finish(),
            View::FormField(n) => f
                .debug_struct("FormField")
                .field("name", &n.name)
                .field("label", &n.label)
                .finish(),
            View::Canvas(n) => f
                .debug_struct("Canvas")
                .field("width", &n.pixel_width)
                .field("height", &n.pixel_height)
                .finish(),
            View::Image(n) => f
                .debug_struct("Image")
                .field("has_data", &n.source.is_some())
                .finish(),
            View::Terminal(n) => f
                .debug_struct("Terminal")
                .field("rows", &n.rows)
                .field("cols", &n.cols)
                .field("border", &n.border)
                .finish(),
            View::ErrorBoundary(_) => f.debug_struct("ErrorBoundary").finish(),
            View::Custom(_) => f.debug_struct("Custom").finish(),
            View::Slider(n) => f
                .debug_struct("Slider")
                .field("min", &n.min)
                .field("max", &n.max)
                .field("value", &n.value)
                .field("step", &n.step)
                .finish(),
            View::Empty => write!(f, "Empty"),
        }
    }
}

impl View {
    /// Create a text view with the given content.
    pub fn text(content: impl Into<String>) -> Self {
        View::Text(TextNode {
            content: content.into(),
            color: None,
            bg_color: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        })
    }

    /// Create a styled text builder.
    pub fn styled_text(content: impl Into<String>) -> TextBuilder {
        TextBuilder::new(content)
    }

    /// Create a vertical stack builder.
    pub fn vstack() -> VStackBuilder {
        VStackBuilder::new()
    }

    /// Create a horizontal stack builder.
    pub fn hstack() -> HStackBuilder {
        HStackBuilder::new()
    }

    /// Create a button builder.
    pub fn button() -> ButtonBuilder {
        ButtonBuilder::new()
    }

    /// Create a box builder.
    pub fn boxed() -> BoxBuilder {
        BoxBuilder::new()
    }

    /// Create a spacer with flex factor 1 (expands to fill available space).
    pub fn spacer() -> Self {
        View::Spacer(SpacerNode { flex: 1, height: 0 })
    }

    /// Create a spacer with a specific flex factor.
    pub fn spacer_flex(flex: u16) -> Self {
        View::Spacer(SpacerNode { flex, height: 0 })
    }

    /// Create a fixed-height gap (blank lines).
    ///
    /// Unlike `spacer()` which expands to fill space, `gap()` is a fixed height.
    ///
    /// # Example
    /// ```rust,ignore
    /// View::vstack()
    ///     .child(View::text("Header"))
    ///     .child(View::gap(1))  // One blank line
    ///     .child(View::text("Content"))
    ///     .build()
    /// ```
    pub fn gap(height: u16) -> Self {
        View::Spacer(SpacerNode { flex: 0, height })
    }

    /// Create a list builder.
    pub fn list() -> ListBuilder {
        ListBuilder::new()
    }

    /// Create a text input builder.
    pub fn text_input() -> TextInputBuilder {
        TextInputBuilder::new()
    }

    /// Create a checkbox builder.
    pub fn checkbox() -> CheckboxBuilder {
        CheckboxBuilder::new()
    }

    /// Create a radio group builder.
    pub fn radio_group() -> RadioGroupBuilder {
        RadioGroupBuilder::new()
    }

    /// Create a text area builder.
    pub fn text_area() -> TextAreaBuilder {
        TextAreaBuilder::new()
    }

    /// Create a modal dialog builder.
    pub fn modal() -> ModalBuilder {
        ModalBuilder::new()
    }

    /// Create a split pane builder.
    pub fn split() -> SplitBuilder {
        SplitBuilder::new()
    }

    /// Create a tabs builder.
    pub fn tabs() -> TabsBuilder {
        TabsBuilder::new()
    }

    /// Create a tree builder.
    pub fn tree() -> TreeBuilder {
        TreeBuilder::new()
    }

    /// Create a table builder.
    pub fn table() -> TableBuilder {
        TableBuilder::new()
    }

    /// Create a progress bar builder.
    pub fn progress_bar() -> ProgressBarBuilder {
        ProgressBarBuilder::new()
    }

    /// Create a status bar builder.
    pub fn status_bar() -> StatusBarBuilder {
        StatusBarBuilder::new()
    }

    /// Create a command palette builder.
    pub fn command_palette() -> CommandPaletteBuilder {
        CommandPaletteBuilder::new()
    }

    /// Create a menu bar builder.
    pub fn menu_bar() -> MenuBarBuilder {
        MenuBarBuilder::new()
    }

    /// Create a toast container builder.
    pub fn toast_container() -> ToastContainerBuilder {
        ToastContainerBuilder::new()
    }

    /// Create a form builder.
    pub fn form() -> FormBuilder {
        FormBuilder::new()
    }

    /// Create a form field builder.
    pub fn form_field(name: impl Into<String>) -> FormFieldBuilder {
        FormFieldBuilder::new(name)
    }

    /// Create a canvas builder for pixel-level drawing.
    ///
    /// **Experimental Feature**
    ///
    /// Canvas uses the Kitty graphics protocol for actual pixel rendering.
    /// Requires a compatible terminal (Kitty, Ghostty, WezTerm).
    /// Other terminals will show a placeholder message.
    pub fn canvas() -> CanvasBuilder {
        CanvasBuilder::new()
    }

    /// Create an image builder for displaying images.
    ///
    /// **Experimental Feature**
    ///
    /// Displays PNG, JPEG, or GIF images using the Kitty graphics protocol.
    /// GIF animations are handled natively by Kitty.
    /// Requires a compatible terminal (Kitty, Ghostty, WezTerm).
    /// Other terminals will show alt text or a placeholder message.
    pub fn image() -> ImageBuilder {
        ImageBuilder::new()
    }

    /// Create a terminal builder for interactive PTY terminal emulation.
    ///
    /// **Status: Experimental Preview**
    ///
    /// Supports running shell commands (bash, vim, htop, etc.) with full
    /// keyboard input and ANSI color/style rendering.
    ///
    /// # Known Limitations
    ///
    /// - No scrollback buffer
    /// - No terminal resize support
    /// - No copy/paste
    /// - No mouse input
    ///
    /// Use for prototyping and experimentation. Breaking changes likely.
    ///
    /// # Example
    /// ```rust,ignore
    /// let terminal = cx.use_terminal();
    /// if !terminal.is_started() {
    ///     terminal.spawn("bash", &[], 80, 24);
    /// }
    /// View::terminal().handle(terminal).build()
    /// ```
    pub fn terminal() -> TerminalBuilder {
        TerminalBuilder::new()
    }

    /// Create a custom widget view.
    ///
    /// Wraps a user-defined `Widget` implementation in a View.
    /// Use this for custom character-cell rendering that can't be
    /// composed from built-in widgets.
    ///
    /// # Example
    /// ```rust,ignore
    /// let my_widget = Rc::new(RefCell::new(MyWidget::new()));
    /// View::custom(my_widget)
    /// ```
    pub fn custom(widget: Rc<RefCell<dyn Widget>>) -> Self {
        View::Custom(CustomNode { widget })
    }

    /// Create a slider builder for bounded numeric values.
    ///
    /// # Example
    /// ```rust,ignore
    /// View::slider()
    ///     .min(0.0)
    ///     .max(127.0)
    ///     .value(64.0)
    ///     .step(1.0)
    ///     .label("Volume")
    ///     .on_change(move |v| vol.set(v))
    ///     .build()
    /// ```
    pub fn slider() -> SliderBuilder {
        SliderBuilder::new()
    }

    /// Create an error boundary builder.
    ///
    /// An error boundary catches panics in its child view and displays
    /// a fallback view instead of crashing the application.
    ///
    /// # Example
    /// ```rust,ignore
    /// View::error_boundary()
    ///     .child(risky_component_view)
    ///     .fallback(View::text("Something went wrong"))
    ///     .build()
    /// ```
    pub fn error_boundary() -> ErrorBoundaryBuilder {
        ErrorBoundaryBuilder::new()
    }

    /// Create an empty view.
    pub fn empty() -> Self {
        View::Empty
    }

    /// Check if this view is focusable.
    pub fn is_focusable(&self) -> bool {
        match self {
            View::Button(_) => true,
            View::Box(node) => node.scroll,
            View::List(_) => true,
            View::TextInput(_) => true,
            View::TextArea(_) => true,
            View::Checkbox(_) => true,
            View::RadioGroup(_) => true, // RadioGroup is focusable for option selection
            View::Split(_) => false,     // Split is a layout container, not focusable itself
            View::Tabs(_) => true,       // Tabs is focusable for tab switching
            View::Tree(_) => true,       // Tree is focusable for navigation
            View::Table(_) => true,      // Table is focusable for row selection
            View::CommandPalette(_) => true, // Command palette captures all input when visible
            View::MenuBar(_) => true,    // Menu bar is focusable for navigation
            View::FormField(_) => true,  // Form fields are focusable for input
            View::Terminal(_) => true,   // Terminal is focusable for PTY input
            View::Slider(_) => true,    // Slider is focusable for value adjustment
            _ => false,
        }
    }

    /// Get the flex factor of this view (for layout).
    pub fn flex(&self) -> u16 {
        match self {
            View::Box(n) => n.flex,
            View::Spacer(n) => n.flex,
            _ => 0,
        }
    }

    /// Get the minimum height constraint, if any.
    pub fn min_height(&self) -> Option<u16> {
        match self {
            View::Box(n) => n.min_height,
            _ => None,
        }
    }

    /// Get the maximum height constraint, if any.
    pub fn max_height(&self) -> Option<u16> {
        match self {
            View::Box(n) => n.max_height,
            _ => None,
        }
    }

    /// Get the minimum width constraint, if any.
    pub fn min_width(&self) -> Option<u16> {
        match self {
            View::Box(n) => n.min_width,
            _ => None,
        }
    }

    /// Get the maximum width constraint, if any.
    pub fn max_width(&self) -> Option<u16> {
        match self {
            View::Box(n) => n.max_width,
            _ => None,
        }
    }

    /// Calculate the intrinsic (natural) height of this view based on its content.
    /// Returns None for views that have no intrinsic height (flexible).
    pub fn intrinsic_height(&self) -> Option<u16> {
        match self {
            View::Text(n) => Some(n.content.lines().count().max(1) as u16),
            View::Button(_) => Some(1),
            View::Box(n) => {
                let border = if n.border { 2 } else { 0 };
                let padding = n.padding * 2;
                let inner = n
                    .child
                    .as_ref()
                    .and_then(|c| c.intrinsic_height())
                    .unwrap_or(0);
                Some(inner + border + padding)
            }
            View::VStack(n) => {
                if n.children.is_empty() {
                    return Some(0);
                }
                let spacing = if n.children.len() > 1 {
                    n.spacing * (n.children.len() as u16 - 1)
                } else {
                    0
                };
                let children_height: u16 =
                    n.children.iter().filter_map(|c| c.intrinsic_height()).sum();
                Some(children_height + spacing)
            }
            View::HStack(n) => {
                // HStack height is max of children heights
                n.children
                    .iter()
                    .filter_map(|c| c.intrinsic_height())
                    .max()
                    .or(Some(1))
            }
            View::List(n) => Some(n.items.len().max(1) as u16),
            View::TextInput(_) => Some(1),
            View::TextArea(n) => Some(n.rows),
            View::Checkbox(_) => Some(1),
            View::RadioGroup(n) => Some(n.options.len() as u16), // One row per option
            View::Modal(_) => None, // Modal is an overlay, no intrinsic size
            View::Spacer(n) => {
                if n.flex == 0 {
                    Some(n.height) // Fixed-height gap
                } else {
                    None // Flexible spacer expands
                }
            }
            View::Split(_) => None,          // Split fills available space
            View::Tabs(_) => None,           // Tabs fills available space
            View::Tree(_) => None,           // Tree fills available space
            View::Table(_) => None,          // Table fills available space
            View::ProgressBar(_) => Some(1), // Progress bar is 1 row tall
            View::StatusBar(_) => Some(1),   // Status bar is 1 row tall
            View::CommandPalette(_) => None, // Command palette is an overlay
            View::MenuBar(_) => Some(1),     // Menu bar is 1 row tall
            View::ToastContainer(_) => None, // Toast container is an overlay
            View::Form(n) => {
                // Form height is sum of children
                let children_height: u16 =
                    n.children.iter().filter_map(|c| c.intrinsic_height()).sum();
                Some(children_height)
            }
            View::FormField(n) => {
                // Label (1) + input (1) + error (1 if present) = 2-3 rows
                let base_height = 2u16; // Label + input
                let error_height = if n.error.is_some() { 1 } else { 0 };
                Some(base_height + error_height)
            }
            View::Canvas(n) => {
                // Canvas height in cells (pixels / cell_height)
                // Approximate: assume ~20 pixels per cell height
                Some((n.pixel_height / 20).max(1))
            }
            View::Image(n) => {
                // Image height based on detected dimensions or default
                n.cell_height.or(Some(5))
            }
            View::Terminal(n) => {
                // Terminal height is rows + border
                let border = if n.border { 2 } else { 0 };
                Some(n.rows as u16 + border)
            }
            View::ErrorBoundary(n) => n.child.intrinsic_height(),
            View::Custom(n) => n.widget.borrow().height_hint(80), // Use default width hint
            View::Slider(_) => Some(1), // Slider is a single row
            View::Empty => Some(0),
        }
    }

    /// Calculate the intrinsic (natural) width of this view based on its content.
    /// Returns None for views that have no intrinsic width (flexible).
    pub fn intrinsic_width(&self) -> Option<u16> {
        match self {
            View::Text(n) => {
                let max_line_width = n.content.lines().map(|l| l.len()).max().unwrap_or(0);
                Some(max_line_width as u16)
            }
            View::Button(n) => {
                // [ label ] = 4 chars for brackets + spaces + label
                Some(n.label.len() as u16 + 4)
            }
            View::Box(n) => {
                let border = if n.border { 2 } else { 0 };
                let padding = n.padding * 2;
                let inner = n
                    .child
                    .as_ref()
                    .and_then(|c| c.intrinsic_width())
                    .unwrap_or(0);
                Some(inner + border + padding)
            }
            View::VStack(n) => {
                // VStack width is max of children widths
                n.children
                    .iter()
                    .filter_map(|c| c.intrinsic_width())
                    .max()
                    .or(Some(1))
            }
            View::HStack(n) => {
                if n.children.is_empty() {
                    return Some(0);
                }
                let spacing = if n.children.len() > 1 {
                    n.spacing * (n.children.len() as u16 - 1)
                } else {
                    0
                };
                let children_width: u16 =
                    n.children.iter().filter_map(|c| c.intrinsic_width()).sum();
                Some(children_width + spacing)
            }
            View::List(n) => {
                // "> " prefix + max item length
                let max_item = n.items.iter().map(|i| i.len()).max().unwrap_or(0);
                Some(max_item as u16 + 2)
            }
            View::TextInput(_) => {
                // TextInput should be sized by its container, not by content
                // Return None to allow flex/container sizing with internal scrolling
                None
            }
            View::TextArea(_) => {
                // TextArea should be sized by its container, not by content
                // Return None to allow flex/container sizing with internal scrolling
                None
            }
            View::Checkbox(n) => {
                // "[x] " + label
                Some(n.label.len() as u16 + 4)
            }
            View::RadioGroup(n) => {
                // "(o) " + longest option
                let max_option = n.options.iter().map(|o| o.len()).max().unwrap_or(0);
                Some(max_option as u16 + 4)
            }
            View::Modal(_) => None, // Modal is an overlay
            View::Spacer(n) => {
                if n.flex == 0 {
                    Some(0) // Fixed-height gap has no width requirement
                } else {
                    None // Flexible spacer expands
                }
            }
            View::Split(_) => None, // Split fills available space
            View::Tabs(_) => None,  // Tabs fills available space
            View::Tree(_) => None,  // Tree fills available space
            View::Table(_) => None, // Table fills available space
            View::ProgressBar(n) => {
                // Label + bar width + percentage
                let label_width = n.label.as_ref().map(|l| l.len() + 1).unwrap_or(0);
                let bar_width = n.width.unwrap_or(10) as usize;
                let percentage_width = if n.show_percentage { 5 } else { 0 };
                Some((label_width + bar_width + percentage_width) as u16)
            }
            View::StatusBar(n) => {
                // Left + center + right sections
                let left_width = n.left.len();
                let center_width = n.center.as_ref().map(|c| c.len()).unwrap_or(0);
                let right_width = n.right.as_ref().map(|r| r.len()).unwrap_or(0);
                // Minimum spacing between sections
                let spacing = if center_width > 0 || right_width > 0 {
                    2
                } else {
                    0
                };
                Some((left_width + center_width + right_width + spacing) as u16)
            }
            View::CommandPalette(_) => None, // Command palette is an overlay
            View::MenuBar(n) => {
                // Sum of menu labels + separators
                let labels_width: usize = n.menus.iter().map(|m| m.label.len() + 3).sum(); // " Label "
                Some(labels_width as u16)
            }
            View::ToastContainer(_) => None, // Toast container is an overlay
            View::Form(n) => {
                // Form width is max of children
                n.children.iter().filter_map(|c| c.intrinsic_width()).max()
            }
            View::FormField(n) => {
                // Width is max of label and input
                let label_width = n.label.len() as u16;
                let input_width = 20u16; // Default minimum input width
                Some(label_width.max(input_width))
            }
            View::Canvas(n) => {
                // Canvas width in cells (pixels / cell_width)
                // Approximate: assume ~10 pixels per cell width
                Some((n.pixel_width / 10).max(1))
            }
            View::Image(n) => {
                // Image width based on detected dimensions or default
                n.cell_width.or(Some(10))
            }
            View::Terminal(n) => {
                // Terminal width is cols + border
                let border = if n.border { 2 } else { 0 };
                Some(n.cols as u16 + border)
            }
            View::ErrorBoundary(n) => n.child.intrinsic_width(),
            View::Custom(n) => n.widget.borrow().width_hint(),
            View::Slider(n) => {
                // Label + brackets + track + value display
                let label_len = n.label.as_ref().map(|l| l.len() + 1).unwrap_or(0) as u16;
                Some(label_len + 20) // Reasonable default width
            }
            View::Empty => Some(0),
        }
    }
}

/// Orientation for split panes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Panes side by side: [first | second]
    #[default]
    Horizontal,
    /// Panes stacked: [first] / [second]
    Vertical,
}

/// Position of the tab bar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TabPosition {
    /// Tab bar at the top (default).
    #[default]
    Top,
    /// Tab bar at the bottom.
    Bottom,
}

/// A text node containing string content with optional styling.
#[derive(Debug, Clone)]
pub struct TextNode {
    pub content: String,
    pub color: Option<crossterm::style::Color>,
    pub bg_color: Option<crossterm::style::Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

/// A vertical stack container.
#[derive(Debug, Clone)]
pub struct VStackNode {
    pub children: Vec<View>,
    /// Spacing between children (in rows).
    pub spacing: u16,
    /// Justify content along main axis (vertical).
    pub justify: Justify,
    /// Align items along cross axis (horizontal).
    pub align: Align,
    /// Layout algorithm to use.
    pub layout_mode: LayoutMode,
}

/// A horizontal stack container.
#[derive(Debug, Clone)]
pub struct HStackNode {
    pub children: Vec<View>,
    /// Spacing between children (in columns).
    pub spacing: u16,
    /// Justify content along main axis (horizontal).
    pub justify: Justify,
    /// Align items along cross axis (vertical).
    pub align: Align,
    /// Layout algorithm to use.
    pub layout_mode: LayoutMode,
}

/// A container with optional border, padding, and flex sizing.
#[derive(Debug, Clone)]
pub struct BoxNode {
    /// The child view inside the box.
    pub child: Option<std::boxed::Box<View>>,
    /// Whether to draw a border around the box.
    pub border: bool,
    /// Padding inside the box (all sides).
    pub padding: u16,
    /// Flex factor for layout (0 = fixed size, >0 = flexible).
    pub flex: u16,
    /// Whether this box is scrollable.
    pub scroll: bool,
    /// Automatically scroll to show bottom content (for chat-like UIs).
    pub auto_scroll_bottom: bool,
    /// Whether this box participates in focus navigation (default: true for scrollable boxes).
    pub focusable: bool,
    /// Minimum width constraint.
    pub min_width: Option<u16>,
    /// Maximum width constraint.
    pub max_width: Option<u16>,
    /// Minimum height constraint.
    pub min_height: Option<u16>,
    /// Maximum height constraint.
    pub max_height: Option<u16>,
}

/// Flexible space that expands to fill available space.
#[derive(Debug, Clone)]
pub struct SpacerNode {
    /// Flex factor (default 1). If 0, uses fixed height.
    pub flex: u16,
    /// Fixed height in rows (only used when flex is 0).
    pub height: u16,
}

/// A button node.
#[derive(Clone)]
pub struct ButtonNode {
    pub label: String,
    pub on_press: Option<Callback>,
}

/// A selectable list node.
#[derive(Clone)]
pub struct ListNode {
    /// The list items to display.
    pub items: Vec<String>,
    /// Currently selected index.
    pub selected: usize,
    /// Callback when selection changes.
    pub on_select: Option<SelectCallback>,
}

/// A text input node.
#[derive(Clone)]
pub struct TextInputNode {
    /// Current text value.
    pub value: String,
    /// Placeholder text shown when empty.
    pub placeholder: String,
    /// Callback when text changes.
    pub on_change: Option<ChangeCallback>,
    /// Callback when cursor position changes.
    pub on_cursor_change: Option<CursorPosCallback>,
    /// Callback when Enter is pressed (submit).
    pub on_submit: Option<Callback>,
    /// Callback when Up arrow is pressed.
    pub on_key_up: Option<Callback>,
    /// Callback when Down arrow is pressed.
    pub on_key_down: Option<Callback>,
    /// Cursor position within the text.
    pub cursor_pos: usize,
    /// Whether this input should have initial focus.
    pub focused: bool,
}

/// A multi-line text area node.
#[derive(Clone)]
pub struct TextAreaNode {
    /// Current text value (may contain newlines).
    pub value: String,
    /// Placeholder text shown when empty.
    pub placeholder: String,
    /// Callback when text changes.
    pub on_change: Option<ChangeCallback>,
    /// Callback when cursor position changes (line, column).
    pub on_cursor_change: Option<CursorChangeCallback>,
    /// Cursor line position.
    pub cursor_line: usize,
    /// Cursor column position.
    pub cursor_col: usize,
    /// Number of visible rows.
    pub rows: u16,
    /// Width at which to auto-wrap text (None = no wrap, text truncated at display edge).
    pub wrap_width: Option<u16>,
}

/// A checkbox node.
#[derive(Clone)]
pub struct CheckboxNode {
    /// Whether the checkbox is checked.
    pub checked: bool,
    /// Label displayed next to the checkbox.
    pub label: String,
    /// Callback when toggled.
    pub on_toggle: Option<ToggleCallback>,
}

/// A radio group node (mutually exclusive options).
#[derive(Clone)]
pub struct RadioGroupNode {
    /// The available options.
    pub options: Vec<String>,
    /// Index of the currently selected option.
    pub selected: usize,
    /// Optional label for the group.
    pub label: Option<String>,
    /// Callback when selection changes.
    pub on_change: Option<SelectCallback>,
}

/// A modal dialog node.
#[derive(Clone)]
pub struct ModalNode {
    /// Whether the modal is visible.
    pub visible: bool,
    /// Title of the modal (shown in border).
    pub title: String,
    /// The content view inside the modal.
    pub child: Option<std::boxed::Box<View>>,
    /// Callback when modal is dismissed (Escape key).
    pub on_dismiss: Option<Callback>,
    /// Width of the modal (percentage of screen, 0-100).
    pub width_percent: u16,
    /// Height of the modal (percentage of screen, 0-100).
    pub height_percent: u16,
}

/// A split pane container node.
#[derive(Clone)]
pub struct SplitNode {
    /// Orientation of the split (horizontal or vertical).
    pub orientation: Orientation,
    /// First pane content.
    pub first: std::boxed::Box<View>,
    /// Second pane content.
    pub second: std::boxed::Box<View>,
    /// Split ratio (0.0 to 1.0, where 0.5 is equal split).
    pub ratio: f32,
    /// Minimum size for first pane (in cells).
    pub min_first: Option<u16>,
    /// Minimum size for second pane (in cells).
    pub min_second: Option<u16>,
    /// Whether to show a divider line between panes.
    pub show_divider: bool,
}

/// A tabbed interface container node.
#[derive(Clone)]
pub struct TabsNode {
    /// Tab labels displayed in the tab bar.
    pub tabs: Vec<String>,
    /// Content views for each tab.
    pub children: Vec<View>,
    /// Currently active tab index.
    pub active: usize,
    /// Callback when tab changes.
    pub on_change: Option<SelectCallback>,
    /// Position of the tab bar (top or bottom).
    pub position: TabPosition,
}

/// A single item in a tree view.
#[derive(Clone, Debug)]
pub struct TreeItem {
    /// Display label for this item.
    pub label: String,
    /// Optional icon displayed before the label.
    pub icon: Option<String>,
    /// Child items (empty for leaf nodes).
    pub children: Vec<TreeItem>,
    /// Whether this node is expanded (showing children).
    pub expanded: bool,
}

impl TreeItem {
    /// Create a new tree item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Set the icon for this item.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Add a child item.
    pub fn child(mut self, child: TreeItem) -> Self {
        self.children.push(child);
        self
    }

    /// Set whether this node is expanded.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Check if this is a leaf node (no children).
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// A hierarchical tree view node.
#[derive(Clone)]
pub struct TreeNode {
    /// Root-level tree items.
    pub items: Vec<TreeItem>,
    /// Path to the currently selected item.
    pub selected: TreePath,
    /// Callback when selection changes.
    pub on_select: Option<TreeSelectCallback>,
    /// Callback when an item is activated (Enter/Space).
    pub on_activate: Option<TreeActivateCallback>,
}

/// Text alignment for table columns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Column width specification for tables.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColumnWidth {
    /// Size to fit content (default).
    #[default]
    Auto,
    /// Fixed width in characters.
    Fixed(u16),
    /// Flex factor for remaining space.
    Flex(u16),
}

/// A column definition for a table.
#[derive(Debug, Clone)]
pub struct TableColumn {
    /// Header text for this column.
    pub header: String,
    /// Width specification.
    pub width: ColumnWidth,
    /// Whether this column is sortable.
    pub sortable: bool,
    /// Text alignment for this column.
    pub align: TextAlign,
}

impl TableColumn {
    /// Create a new table column with the given header.
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            width: ColumnWidth::Auto,
            sortable: false,
            align: TextAlign::Left,
        }
    }

    /// Set the width of this column.
    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    /// Make this column sortable.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Set the text alignment for this column.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

/// A data table node with columns and rows.
#[derive(Clone)]
pub struct TableNode {
    /// Column definitions.
    pub columns: Vec<TableColumn>,
    /// Row data (each row is a Vec of cell strings).
    pub rows: Vec<Vec<String>>,
    /// Currently selected row index.
    pub selected: usize,
    /// Current sort state: (column_index, ascending).
    pub sort: Option<(usize, bool)>,
    /// Callback when selection changes.
    pub on_select: Option<SelectCallback>,
    /// Callback when sort changes.
    pub on_sort: Option<SortCallback>,
    /// Callback when a row is activated (Enter).
    pub on_activate: Option<RowActivateCallback>,
}

/// A progress bar node.
#[derive(Clone)]
pub struct ProgressBarNode {
    /// Progress value from 0.0 to 1.0.
    pub value: f32,
    /// Optional label shown before the bar.
    pub label: Option<String>,
    /// Whether to show percentage after the bar.
    pub show_percentage: bool,
    /// Fixed width of the bar portion (None = expand to fill).
    pub width: Option<u16>,
    /// Character used for the filled portion.
    pub filled_char: char,
    /// Character used for the empty portion.
    pub empty_char: char,
}

/// A status bar node displayed at the bottom of the screen.
#[derive(Clone)]
pub struct StatusBarNode {
    /// Content for the left section.
    pub left: String,
    /// Content for the center section (optional).
    pub center: Option<String>,
    /// Content for the right section (optional).
    pub right: Option<String>,
    /// Background color for the status bar.
    pub bg_color: Option<crossterm::style::Color>,
    /// Foreground color for the status bar.
    pub fg_color: Option<crossterm::style::Color>,
}

/// Builder for VStack views.
#[derive(Debug, Default)]
pub struct VStackBuilder {
    children: Vec<View>,
    spacing: u16,
    justify: Justify,
    align: Align,
    layout_mode: LayoutMode,
}

impl VStackBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, view: View) -> Self {
        self.children.push(view);
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set justify (main axis alignment for VStack = vertical).
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Set align (cross axis alignment for VStack = horizontal).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Set layout mode (algorithm for distributing space).
    pub fn layout_mode(mut self, mode: LayoutMode) -> Self {
        self.layout_mode = mode;
        self
    }

    pub fn build(self) -> View {
        View::VStack(VStackNode {
            children: self.children,
            spacing: self.spacing,
            justify: self.justify,
            align: self.align,
            layout_mode: self.layout_mode,
        })
    }
}

/// Builder for HStack views.
#[derive(Debug, Default)]
pub struct HStackBuilder {
    children: Vec<View>,
    spacing: u16,
    justify: Justify,
    align: Align,
    layout_mode: LayoutMode,
}

impl HStackBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, view: View) -> Self {
        self.children.push(view);
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set justify (main axis alignment for HStack = horizontal).
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Set align (cross axis alignment for HStack = vertical).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Set layout mode (algorithm for distributing space).
    pub fn layout_mode(mut self, mode: LayoutMode) -> Self {
        self.layout_mode = mode;
        self
    }

    pub fn build(self) -> View {
        View::HStack(HStackNode {
            children: self.children,
            spacing: self.spacing,
            justify: self.justify,
            align: self.align,
            layout_mode: self.layout_mode,
        })
    }
}

/// Builder for Button views.
#[derive(Default)]
pub struct ButtonBuilder {
    label: String,
    on_press: Option<Callback>,
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn on_press(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_press = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::Button(ButtonNode {
            label: self.label,
            on_press: self.on_press,
        })
    }
}

/// Builder for Box views.
#[derive(Default)]
pub struct BoxBuilder {
    child: Option<View>,
    border: bool,
    padding: u16,
    flex: u16,
    scroll: bool,
    auto_scroll_bottom: bool,
    focusable: Option<bool>,
    min_width: Option<u16>,
    max_width: Option<u16>,
    min_height: Option<u16>,
    max_height: Option<u16>,
}

impl BoxBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, view: View) -> Self {
        self.child = Some(view);
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn flex(mut self, flex: u16) -> Self {
        self.flex = flex;
        self
    }

    pub fn scroll(mut self, scroll: bool) -> Self {
        self.scroll = scroll;
        self
    }

    /// Enable auto-scrolling to bottom (for chat-like UIs).
    pub fn auto_scroll_bottom(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll_bottom = auto_scroll;
        self
    }

    /// Set whether this box participates in focus navigation.
    /// By default, scrollable boxes are focusable. Use `focusable(false)` to
    /// disable focus for a scrollable box (e.g., auto-scroll chat messages).
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = Some(focusable);
        self
    }

    pub fn min_width(mut self, width: u16) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn max_width(mut self, width: u16) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn min_height(mut self, height: u16) -> Self {
        self.min_height = Some(height);
        self
    }

    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn build(self) -> View {
        // Scrollable boxes are focusable by default so users can scroll back
        let default_focusable = self.scroll || self.auto_scroll_bottom;
        View::Box(BoxNode {
            child: self.child.map(std::boxed::Box::new),
            border: self.border,
            padding: self.padding,
            flex: self.flex,
            scroll: self.scroll,
            auto_scroll_bottom: self.auto_scroll_bottom,
            focusable: self.focusable.unwrap_or(default_focusable),
            min_width: self.min_width,
            max_width: self.max_width,
            min_height: self.min_height,
            max_height: self.max_height,
        })
    }
}

/// Builder for List views.
#[derive(Default)]
pub struct ListBuilder {
    items: Vec<String>,
    selected: usize,
    on_select: Option<SelectCallback>,
}

impl ListBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_select(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::List(ListNode {
            items: self.items,
            selected: self.selected,
            on_select: self.on_select,
        })
    }
}

/// Builder for TextInput views.
#[derive(Default)]
pub struct TextInputBuilder {
    value: String,
    placeholder: String,
    on_change: Option<ChangeCallback>,
    on_cursor_change: Option<CursorPosCallback>,
    on_submit: Option<Callback>,
    on_key_up: Option<Callback>,
    on_key_down: Option<Callback>,
    cursor_pos: usize,
    focused: bool,
}

impl TextInputBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        // Default cursor to end of value
        self.cursor_pos = self.value.len();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    pub fn on_cursor_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_cursor_change = Some(Rc::new(callback));
        self
    }

    pub fn on_submit(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_submit = Some(Rc::new(callback));
        self
    }

    /// Set callback for when Up arrow is pressed (e.g., for command history).
    pub fn on_key_up(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_key_up = Some(Rc::new(callback));
        self
    }

    /// Set callback for when Down arrow is pressed (e.g., for command history).
    pub fn on_key_down(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_key_down = Some(Rc::new(callback));
        self
    }

    pub fn cursor(mut self, pos: usize) -> Self {
        self.cursor_pos = pos;
        self
    }

    /// Set this input to have initial focus when the app starts.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn build(self) -> View {
        View::TextInput(TextInputNode {
            value: self.value.clone(),
            placeholder: self.placeholder,
            on_change: self.on_change,
            on_cursor_change: self.on_cursor_change,
            on_submit: self.on_submit,
            on_key_up: self.on_key_up,
            on_key_down: self.on_key_down,
            cursor_pos: self.cursor_pos.min(self.value.len()),
            focused: self.focused,
        })
    }
}

/// Builder for Checkbox views.
#[derive(Default)]
pub struct CheckboxBuilder {
    checked: bool,
    label: String,
    on_toggle: Option<ToggleCallback>,
}

impl CheckboxBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn on_toggle(mut self, callback: impl Fn(bool) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::Checkbox(CheckboxNode {
            checked: self.checked,
            label: self.label,
            on_toggle: self.on_toggle,
        })
    }
}

/// Builder for RadioGroup views.
#[derive(Default)]
pub struct RadioGroupBuilder {
    options: Vec<String>,
    selected: usize,
    label: Option<String>,
    on_change: Option<SelectCallback>,
}

impl RadioGroupBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the available options.
    pub fn options(mut self, options: Vec<impl Into<String>>) -> Self {
        self.options = options.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add a single option.
    pub fn option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Set the currently selected option index.
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    /// Set an optional label for the group.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the callback when selection changes.
    pub fn on_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::RadioGroup(RadioGroupNode {
            options: self.options,
            selected: self.selected,
            label: self.label,
            on_change: self.on_change,
        })
    }
}

/// Builder for styled Text views.
#[derive(Debug, Default)]
pub struct TextBuilder {
    content: String,
    color: Option<crossterm::style::Color>,
    bg_color: Option<crossterm::style::Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    dim: bool,
}

impl TextBuilder {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    /// Set the text color.
    pub fn color(mut self, color: crossterm::style::Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the background color.
    pub fn bg(mut self, color: crossterm::style::Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    /// Make the text bold.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Make the text italic.
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Underline the text.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Make the text dim/faded.
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    pub fn build(self) -> View {
        View::Text(TextNode {
            content: self.content,
            color: self.color,
            bg_color: self.bg_color,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            dim: self.dim,
        })
    }
}

/// Builder for TextArea views.
#[derive(Default)]
pub struct TextAreaBuilder {
    value: String,
    placeholder: String,
    on_change: Option<ChangeCallback>,
    on_cursor_change: Option<CursorChangeCallback>,
    cursor_line: usize,
    cursor_col: usize,
    rows: u16,
    wrap_width: Option<u16>,
}

impl TextAreaBuilder {
    pub fn new() -> Self {
        Self {
            rows: 5, // Default to 5 rows
            ..Default::default()
        }
    }

    /// Set the current text value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set the placeholder text shown when empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the callback for when text changes.
    pub fn on_change(mut self, callback: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Set the callback for when cursor position changes.
    pub fn on_cursor_change(mut self, callback: impl Fn(usize, usize) + 'static) -> Self {
        self.on_cursor_change = Some(Rc::new(callback));
        self
    }

    /// Set the cursor line position.
    pub fn cursor_line(mut self, line: usize) -> Self {
        self.cursor_line = line;
        self
    }

    /// Set the cursor column position.
    pub fn cursor_col(mut self, col: usize) -> Self {
        self.cursor_col = col;
        self
    }

    /// Set the number of visible rows.
    pub fn rows(mut self, rows: u16) -> Self {
        self.rows = rows;
        self
    }

    /// Set the width at which text automatically wraps to the next line.
    /// If not set, text is truncated at the display edge without wrapping.
    pub fn wrap_width(mut self, width: u16) -> Self {
        self.wrap_width = Some(width);
        self
    }

    pub fn build(self) -> View {
        View::TextArea(TextAreaNode {
            value: self.value,
            placeholder: self.placeholder,
            on_change: self.on_change,
            on_cursor_change: self.on_cursor_change,
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
            rows: self.rows,
            wrap_width: self.wrap_width,
        })
    }
}

/// Builder for Modal views.
#[derive(Default)]
pub struct ModalBuilder {
    visible: bool,
    title: String,
    child: Option<View>,
    on_dismiss: Option<Callback>,
    width_percent: u16,
    height_percent: u16,
}

impl ModalBuilder {
    pub fn new() -> Self {
        Self {
            width_percent: 60,
            height_percent: 50,
            ..Default::default()
        }
    }

    /// Set whether the modal is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the title shown in the modal border.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the content of the modal.
    pub fn child(mut self, view: View) -> Self {
        self.child = Some(view);
        self
    }

    /// Set the callback when modal is dismissed (Escape key).
    pub fn on_dismiss(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(callback));
        self
    }

    /// Set the width as percentage of screen (0-100).
    pub fn width(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Set the height as percentage of screen (0-100).
    pub fn height(mut self, percent: u16) -> Self {
        self.height_percent = percent.min(100);
        self
    }

    pub fn build(self) -> View {
        View::Modal(ModalNode {
            visible: self.visible,
            title: self.title,
            child: self.child.map(std::boxed::Box::new),
            on_dismiss: self.on_dismiss,
            width_percent: self.width_percent,
            height_percent: self.height_percent,
        })
    }
}

/// Builder for Split pane views.
#[derive(Default)]
pub struct SplitBuilder {
    orientation: Orientation,
    first: Option<View>,
    second: Option<View>,
    ratio: f32,
    min_first: Option<u16>,
    min_second: Option<u16>,
    show_divider: bool,
}

impl SplitBuilder {
    pub fn new() -> Self {
        Self {
            ratio: 0.5, // Default to equal split
            show_divider: true,
            ..Default::default()
        }
    }

    /// Set the orientation to horizontal (side by side).
    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }

    /// Set the orientation to vertical (stacked).
    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    /// Set the first pane content.
    pub fn first(mut self, view: View) -> Self {
        self.first = Some(view);
        self
    }

    /// Set the second pane content.
    pub fn second(mut self, view: View) -> Self {
        self.second = Some(view);
        self
    }

    /// Set the split ratio (0.0 to 1.0, where 0.5 is equal split).
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Set the minimum size for the first pane (in cells).
    pub fn min_first(mut self, min: u16) -> Self {
        self.min_first = Some(min);
        self
    }

    /// Set the minimum size for the second pane (in cells).
    pub fn min_second(mut self, min: u16) -> Self {
        self.min_second = Some(min);
        self
    }

    /// Set whether to show a divider line between panes.
    pub fn show_divider(mut self, show: bool) -> Self {
        self.show_divider = show;
        self
    }

    pub fn build(self) -> View {
        View::Split(SplitNode {
            orientation: self.orientation,
            first: std::boxed::Box::new(self.first.unwrap_or(View::Empty)),
            second: std::boxed::Box::new(self.second.unwrap_or(View::Empty)),
            ratio: self.ratio,
            min_first: self.min_first,
            min_second: self.min_second,
            show_divider: self.show_divider,
        })
    }
}

/// Builder for Tabs views.
#[derive(Default)]
pub struct TabsBuilder {
    tabs: Vec<String>,
    children: Vec<View>,
    active: usize,
    on_change: Option<SelectCallback>,
    position: TabPosition,
}

impl TabsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tab with a label and content view.
    pub fn tab(mut self, label: impl Into<String>, content: View) -> Self {
        self.tabs.push(label.into());
        self.children.push(content);
        self
    }

    /// Set the active tab index.
    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    /// Set the callback when tab changes.
    pub fn on_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Set the position of the tab bar.
    pub fn position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    pub fn build(self) -> View {
        View::Tabs(TabsNode {
            tabs: self.tabs,
            children: self.children,
            active: self.active,
            on_change: self.on_change,
            position: self.position,
        })
    }
}

/// Builder for Tree views.
#[derive(Default)]
pub struct TreeBuilder {
    items: Vec<TreeItem>,
    selected: TreePath,
    on_select: Option<TreeSelectCallback>,
    on_activate: Option<TreeActivateCallback>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tree items.
    pub fn items(mut self, items: Vec<TreeItem>) -> Self {
        self.items = items;
        self
    }

    /// Add a single root item.
    pub fn item(mut self, item: TreeItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the selected path.
    pub fn selected(mut self, path: TreePath) -> Self {
        self.selected = path;
        self
    }

    /// Set the callback when selection changes.
    pub fn on_select(mut self, callback: impl Fn(TreePath) + 'static) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }

    /// Set the callback when an item is activated.
    pub fn on_activate(mut self, callback: impl Fn(TreePath) + 'static) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::Tree(TreeNode {
            items: self.items,
            selected: self.selected,
            on_select: self.on_select,
            on_activate: self.on_activate,
        })
    }
}

/// Builder for Table views.
#[derive(Default)]
pub struct TableBuilder {
    columns: Vec<TableColumn>,
    rows: Vec<Vec<String>>,
    selected: usize,
    sort: Option<(usize, bool)>,
    on_select: Option<SelectCallback>,
    on_sort: Option<SortCallback>,
    on_activate: Option<RowActivateCallback>,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a column with just a header (auto width, left aligned, not sortable).
    pub fn column(mut self, header: impl Into<String>) -> Self {
        self.columns.push(TableColumn::new(header));
        self
    }

    /// Add a column with full configuration.
    pub fn column_with(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Set the row data.
    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    /// Add a single row.
    pub fn row(mut self, row: Vec<String>) -> Self {
        self.rows.push(row);
        self
    }

    /// Set the selected row index.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Set the sort state (column index, ascending).
    pub fn sort(mut self, sort: Option<(usize, bool)>) -> Self {
        self.sort = sort;
        self
    }

    /// Set the sort state with explicit column and direction.
    pub fn sort_by(mut self, column: usize, ascending: bool) -> Self {
        self.sort = Some((column, ascending));
        self
    }

    /// Set the callback when selection changes.
    pub fn on_select(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }

    /// Set the callback when sort changes.
    pub fn on_sort(mut self, callback: impl Fn(usize, bool) + 'static) -> Self {
        self.on_sort = Some(Rc::new(callback));
        self
    }

    /// Set the callback when a row is activated.
    pub fn on_activate(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::Table(TableNode {
            columns: self.columns,
            rows: self.rows,
            selected: self.selected,
            sort: self.sort,
            on_select: self.on_select,
            on_sort: self.on_sort,
            on_activate: self.on_activate,
        })
    }
}

/// Builder for ProgressBar views.
#[derive(Debug, Clone)]
pub struct ProgressBarBuilder {
    value: f32,
    label: Option<String>,
    show_percentage: bool,
    width: Option<u16>,
    filled_char: char,
    empty_char: char,
}

impl Default for ProgressBarBuilder {
    fn default() -> Self {
        Self {
            value: 0.0,
            label: None,
            show_percentage: true,
            width: None,
            filled_char: '█',
            empty_char: '░',
        }
    }
}

impl ProgressBarBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the progress value (0.0 to 1.0).
    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    /// Set a label shown before the bar.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set whether to show percentage after the bar (default: true).
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set a fixed width for the bar portion.
    /// If not set, the bar expands to fill available space.
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the character used for the filled portion (default: █).
    pub fn filled_char(mut self, ch: char) -> Self {
        self.filled_char = ch;
        self
    }

    /// Set the character used for the empty portion (default: ░).
    pub fn empty_char(mut self, ch: char) -> Self {
        self.empty_char = ch;
        self
    }

    pub fn build(self) -> View {
        View::ProgressBar(ProgressBarNode {
            value: self.value,
            label: self.label,
            show_percentage: self.show_percentage,
            width: self.width,
            filled_char: self.filled_char,
            empty_char: self.empty_char,
        })
    }
}

/// Builder for StatusBar views.
#[derive(Debug, Clone, Default)]
pub struct StatusBarBuilder {
    left: String,
    center: Option<String>,
    right: Option<String>,
    bg_color: Option<crossterm::style::Color>,
    fg_color: Option<crossterm::style::Color>,
}

impl StatusBarBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the left section content.
    pub fn left(mut self, content: impl Into<String>) -> Self {
        self.left = content.into();
        self
    }

    /// Set the center section content.
    pub fn center(mut self, content: impl Into<String>) -> Self {
        self.center = Some(content.into());
        self
    }

    /// Set the right section content.
    pub fn right(mut self, content: impl Into<String>) -> Self {
        self.right = Some(content.into());
        self
    }

    /// Set the background color.
    pub fn bg(mut self, color: crossterm::style::Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    /// Set the foreground (text) color.
    pub fn fg(mut self, color: crossterm::style::Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn build(self) -> View {
        View::StatusBar(StatusBarNode {
            left: self.left,
            center: self.center,
            right: self.right,
            bg_color: self.bg_color,
            fg_color: self.fg_color,
        })
    }
}

// =============================================================================
// Command Palette
// =============================================================================

/// A command in the command palette.
#[derive(Clone)]
pub struct PaletteCommand {
    /// Unique identifier for the command.
    pub id: &'static str,
    /// Display label.
    pub label: String,
    /// Optional keyboard shortcut display (e.g., "Ctrl+S").
    pub shortcut: Option<String>,
    /// Optional category for grouping.
    pub category: Option<String>,
}

impl PaletteCommand {
    /// Create a new palette command.
    pub fn new(id: &'static str, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            shortcut: None,
            category: None,
        }
    }

    /// Set the shortcut display string.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set the category.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// A command palette overlay for searching and executing commands.
#[derive(Clone)]
pub struct CommandPaletteNode {
    /// Whether the palette is visible.
    pub visible: bool,
    /// Current search query.
    pub query: String,
    /// Available commands.
    pub commands: Vec<PaletteCommand>,
    /// Currently selected index in the filtered list.
    pub selected: usize,
    /// Callback when query changes.
    pub on_query_change: Option<ChangeCallback>,
    /// Callback when a command is selected (receives command ID).
    pub on_select: Option<CommandCallback>,
    /// Callback when the palette is dismissed.
    pub on_dismiss: Option<Callback>,
    /// Width percentage (0-100).
    pub width_percent: u16,
    /// Height percentage (0-100).
    pub height_percent: u16,
}

/// Builder for CommandPalette views.
#[derive(Default)]
pub struct CommandPaletteBuilder {
    visible: bool,
    query: String,
    commands: Vec<PaletteCommand>,
    selected: usize,
    on_query_change: Option<ChangeCallback>,
    on_select: Option<CommandCallback>,
    on_dismiss: Option<Callback>,
    width_percent: u16,
    height_percent: u16,
}

impl CommandPaletteBuilder {
    pub fn new() -> Self {
        Self {
            width_percent: 50,
            height_percent: 60,
            ..Default::default()
        }
    }

    /// Set whether the palette is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the current query.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    /// Set the available commands.
    pub fn commands(mut self, commands: Vec<PaletteCommand>) -> Self {
        self.commands = commands;
        self
    }

    /// Add a single command.
    pub fn command(mut self, command: PaletteCommand) -> Self {
        self.commands.push(command);
        self
    }

    /// Set the selected index.
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    /// Set the callback for query changes.
    pub fn on_query_change(mut self, callback: impl Fn(String) + 'static) -> Self {
        self.on_query_change = Some(Rc::new(callback));
        self
    }

    /// Set the callback for command selection.
    pub fn on_select(mut self, callback: impl Fn(&'static str) + 'static) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }

    /// Set the callback when dismissed.
    pub fn on_dismiss(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(callback));
        self
    }

    /// Set the width as percentage of screen.
    pub fn width(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Set the height as percentage of screen.
    pub fn height(mut self, percent: u16) -> Self {
        self.height_percent = percent.min(100);
        self
    }

    pub fn build(self) -> View {
        View::CommandPalette(CommandPaletteNode {
            visible: self.visible,
            query: self.query,
            commands: self.commands,
            selected: self.selected,
            on_query_change: self.on_query_change,
            on_select: self.on_select,
            on_dismiss: self.on_dismiss,
            width_percent: self.width_percent,
            height_percent: self.height_percent,
        })
    }
}

// =============================================================================
// Menu Bar
// =============================================================================

/// A menu in the menu bar.
#[derive(Clone)]
pub struct Menu {
    /// Display label for the menu.
    pub label: String,
    /// Items in this menu.
    pub items: Vec<MenuItemNode>,
}

impl Menu {
    /// Create a new menu with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
        }
    }

    /// Add an item to the menu.
    pub fn item(mut self, item: MenuItemNode) -> Self {
        self.items.push(item);
        self
    }

    /// Add a command item.
    pub fn command(self, id: &'static str, label: impl Into<String>) -> Self {
        self.item(MenuItemNode::Command {
            id,
            label: label.into(),
            shortcut: None,
        })
    }

    /// Add a command item with shortcut display.
    pub fn command_with_shortcut(
        self,
        id: &'static str,
        label: impl Into<String>,
        shortcut: impl Into<String>,
    ) -> Self {
        self.item(MenuItemNode::Command {
            id,
            label: label.into(),
            shortcut: Some(shortcut.into()),
        })
    }

    /// Add a separator.
    pub fn separator(self) -> Self {
        self.item(MenuItemNode::Separator)
    }
}

/// An item in a menu.
#[derive(Clone)]
pub enum MenuItemNode {
    /// A command with ID, label, and optional shortcut display.
    Command {
        id: &'static str,
        label: String,
        shortcut: Option<String>,
    },
    /// A visual separator.
    Separator,
}

/// A horizontal menu bar with dropdown menus.
#[derive(Clone)]
pub struct MenuBarNode {
    /// The menus in the menu bar.
    pub menus: Vec<Menu>,
    /// Currently active (open) menu index, if any.
    pub active_menu: Option<usize>,
    /// Currently highlighted menu index (for keyboard navigation when no menu is open).
    pub highlighted_menu: usize,
    /// Currently selected item in the active menu.
    pub selected_item: usize,
    /// Callback when a command is selected.
    pub on_select: Option<CommandCallback>,
    /// Callback when the active menu changes (opens/closes).
    pub on_menu_change: Option<SelectCallback>,
    /// Callback when the highlighted menu changes (arrow key navigation).
    pub on_highlight_change: Option<SelectCallback>,
    /// Callback when the selected item within a menu changes.
    pub on_item_change: Option<SelectCallback>,
}

/// Builder for MenuBar views.
#[derive(Default)]
pub struct MenuBarBuilder {
    menus: Vec<Menu>,
    active_menu: Option<usize>,
    highlighted_menu: usize,
    selected_item: usize,
    on_select: Option<CommandCallback>,
    on_menu_change: Option<SelectCallback>,
    on_highlight_change: Option<SelectCallback>,
    on_item_change: Option<SelectCallback>,
}

impl MenuBarBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a menu to the menu bar.
    pub fn menu(mut self, menu: Menu) -> Self {
        self.menus.push(menu);
        self
    }

    /// Set the active menu index (which menu has its dropdown open).
    pub fn active_menu(mut self, index: Option<usize>) -> Self {
        self.active_menu = index;
        self
    }

    /// Set the highlighted menu index (for keyboard navigation).
    pub fn highlighted_menu(mut self, index: usize) -> Self {
        self.highlighted_menu = index;
        self
    }

    /// Set the selected item in the active menu.
    pub fn selected_item(mut self, index: usize) -> Self {
        self.selected_item = index;
        self
    }

    /// Set the callback for command selection.
    pub fn on_select(mut self, callback: impl Fn(&'static str) + 'static) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }

    /// Set the callback for menu changes (opens/closes dropdown).
    pub fn on_menu_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_menu_change = Some(Rc::new(callback));
        self
    }

    /// Set the callback for highlight changes (arrow key navigation).
    pub fn on_highlight_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_highlight_change = Some(Rc::new(callback));
        self
    }

    /// Set the callback for item selection changes within a menu.
    pub fn on_item_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_item_change = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::MenuBar(MenuBarNode {
            menus: self.menus,
            active_menu: self.active_menu,
            highlighted_menu: self.highlighted_menu,
            selected_item: self.selected_item,
            on_select: self.on_select,
            on_menu_change: self.on_menu_change,
            on_highlight_change: self.on_highlight_change,
            on_item_change: self.on_item_change,
        })
    }
}

// =============================================================================
// Toast Container
// =============================================================================

/// Position for toast notifications.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastPosition {
    /// Top-right corner.
    TopRight,
    /// Top-left corner.
    TopLeft,
    /// Bottom-right corner (default).
    #[default]
    BottomRight,
    /// Bottom-left corner.
    BottomLeft,
}

/// Severity level for visual rendering of toasts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastLevelView {
    /// Informational (default).
    #[default]
    Info,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Error.
    Error,
}

/// A toast item for rendering.
#[derive(Clone)]
pub struct ToastItem {
    /// The message to display.
    pub message: String,
    /// Severity level.
    pub level: ToastLevelView,
    /// Progress (0.0 to 1.0) for fade-out animation.
    pub progress: f32,
}

/// A container for displaying toast notifications.
#[derive(Clone)]
pub struct ToastContainerNode {
    /// The toasts to display.
    pub toasts: Vec<ToastItem>,
    /// Position of the toast container.
    pub position: ToastPosition,
    /// Maximum number of visible toasts.
    pub max_visible: usize,
    /// Width of each toast (in characters).
    pub width: u16,
}

/// Builder for ToastContainer views.
#[derive(Default)]
pub struct ToastContainerBuilder {
    toasts: Vec<ToastItem>,
    position: ToastPosition,
    max_visible: usize,
    width: u16,
}

impl ToastContainerBuilder {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            position: ToastPosition::BottomRight,
            max_visible: 5,
            width: 40,
        }
    }

    /// Set the toasts to display.
    pub fn toasts(mut self, toasts: Vec<ToastItem>) -> Self {
        self.toasts = toasts;
        self
    }

    /// Add a toast from the toast queue system.
    pub fn from_queue(mut self, queue: &crate::toast::ToastQueue) -> Self {
        let toasts = queue.collect();
        self.toasts = toasts
            .into_iter()
            .map(|t| {
                let progress = t.remaining_fraction();
                let level = match t.level {
                    crate::toast::ToastLevel::Info => ToastLevelView::Info,
                    crate::toast::ToastLevel::Success => ToastLevelView::Success,
                    crate::toast::ToastLevel::Warning => ToastLevelView::Warning,
                    crate::toast::ToastLevel::Error => ToastLevelView::Error,
                };
                ToastItem {
                    message: t.message,
                    level,
                    progress,
                }
            })
            .collect();
        self
    }

    /// Set the position of the toast container.
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the maximum number of visible toasts.
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Set the width of each toast.
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn build(self) -> View {
        View::ToastContainer(ToastContainerNode {
            toasts: self.toasts,
            position: self.position,
            max_visible: self.max_visible,
            width: self.width,
        })
    }
}

// =============================================================================
// Form
// =============================================================================

/// Callback type for form submission (receives all field values).
pub type FormSubmitCallback = Rc<dyn Fn(std::collections::HashMap<String, String>)>;

/// A form container that manages field validation.
#[derive(Clone)]
pub struct FormNode {
    /// Child views (typically FormField nodes).
    pub children: Vec<View>,
    /// Callback when form is submitted (all fields valid).
    pub on_submit: Option<FormSubmitCallback>,
    /// Spacing between children.
    pub spacing: u16,
}

/// Builder for Form views.
#[derive(Default)]
pub struct FormBuilder {
    children: Vec<View>,
    on_submit: Option<FormSubmitCallback>,
    spacing: u16,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self {
            spacing: 1,
            ..Default::default()
        }
    }

    /// Add a child view (typically a FormField).
    pub fn child(mut self, view: View) -> Self {
        self.children.push(view);
        self
    }

    /// Set spacing between children.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the submit callback.
    pub fn on_submit(
        mut self,
        callback: impl Fn(std::collections::HashMap<String, String>) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::Form(FormNode {
            children: self.children,
            on_submit: self.on_submit,
            spacing: self.spacing,
        })
    }
}

// =============================================================================
// Form Field
// =============================================================================

/// A form field with label, input, and error display.
#[derive(Clone)]
pub struct FormFieldNode {
    /// Field name (identifier).
    pub name: String,
    /// Display label.
    pub label: String,
    /// Current value.
    pub value: String,
    /// Placeholder text.
    pub placeholder: String,
    /// Error message (if validation failed).
    pub error: Option<String>,
    /// Whether this is a password field (mask input).
    pub password: bool,
    /// Callback when value changes.
    pub on_change: Option<ChangeCallback>,
    /// Callback when field loses focus (for validation).
    pub on_blur: Option<Callback>,
    /// Cursor position.
    pub cursor_pos: usize,
}

/// Builder for FormField views.
#[derive(Default)]
pub struct FormFieldBuilder {
    name: String,
    label: String,
    value: String,
    placeholder: String,
    error: Option<String>,
    password: bool,
    on_change: Option<ChangeCallback>,
    on_blur: Option<Callback>,
    cursor_pos: usize,
}

impl FormFieldBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            label: name.clone(),
            name,
            ..Default::default()
        }
    }

    /// Set the display label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the current value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_pos = self.value.len();
        self
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the error message.
    pub fn error(mut self, error: Option<String>) -> Self {
        self.error = error;
        self
    }

    /// Set whether this is a password field.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Set the change callback.
    pub fn on_change(mut self, callback: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Set the blur callback.
    pub fn on_blur(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_blur = Some(Rc::new(callback));
        self
    }

    /// Set the cursor position.
    pub fn cursor(mut self, pos: usize) -> Self {
        self.cursor_pos = pos;
        self
    }

    pub fn build(self) -> View {
        View::FormField(FormFieldNode {
            name: self.name,
            label: self.label,
            value: self.value.clone(),
            placeholder: self.placeholder,
            error: self.error,
            password: self.password,
            on_change: self.on_change,
            on_blur: self.on_blur,
            cursor_pos: self.cursor_pos.min(self.value.len()),
        })
    }
}

// =============================================================================
// Canvas Widget
// =============================================================================

/// A canvas node for pixel-level drawing using Kitty graphics protocol.
#[derive(Clone)]
pub struct CanvasNode {
    /// Width in pixels.
    pub pixel_width: u16,
    /// Height in pixels.
    pub pixel_height: u16,
    /// Callback to draw on the canvas.
    pub on_draw: Option<CanvasDrawCallback>,
    /// Unique ID for this canvas (for Kitty image caching).
    pub id: u32,
}

/// Builder for Canvas views.
pub struct CanvasBuilder {
    pixel_width: u16,
    pixel_height: u16,
    on_draw: Option<CanvasDrawCallback>,
    id: u32,
}

impl Default for CanvasBuilder {
    fn default() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);

        Self {
            pixel_width: 100,
            pixel_height: 50,
            on_draw: None,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl CanvasBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the canvas width in pixels.
    pub fn width(mut self, width: u16) -> Self {
        self.pixel_width = width;
        self
    }

    /// Set the canvas height in pixels.
    pub fn height(mut self, height: u16) -> Self {
        self.pixel_height = height;
        self
    }

    /// Set the draw callback.
    ///
    /// This callback receives a `DrawContext` and is called each render
    /// to draw the canvas content.
    pub fn on_draw<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut crate::canvas::DrawContext) + 'static,
    {
        self.on_draw = Some(Rc::new(callback));
        self
    }

    /// Set a specific canvas ID (for manual caching control).
    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub fn build(self) -> View {
        View::Canvas(CanvasNode {
            pixel_width: self.pixel_width,
            pixel_height: self.pixel_height,
            on_draw: self.on_draw,
            id: self.id,
        })
    }
}

// =============================================================================
// Image Widget
// =============================================================================

/// An image node for displaying images using Kitty graphics protocol.
#[derive(Clone)]
pub struct ImageNode {
    /// Image data source (bytes or file path).
    pub source: Option<crate::image::ImageSource>,
    /// Unique ID for this image (for Kitty caching).
    pub id: u32,
    /// Explicit width in cells (overrides auto-detection).
    pub cell_width: Option<u16>,
    /// Explicit height in cells (overrides auto-detection).
    pub cell_height: Option<u16>,
    /// Alt text for accessibility / fallback display.
    pub alt: Option<String>,
}

/// Builder for Image views.
pub struct ImageBuilder {
    source: Option<crate::image::ImageSource>,
    id: u32,
    cell_width: Option<u16>,
    cell_height: Option<u16>,
    alt: Option<String>,
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self {
            source: None,
            id: crate::image::next_image_id(),
            cell_width: None,
            cell_height: None,
            alt: None,
        }
    }
}

impl ImageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the image data from raw bytes.
    ///
    /// Supports PNG, JPEG, and GIF formats. Kitty auto-detects the format.
    /// GIF animations are handled natively by Kitty.
    ///
    /// # Example
    /// ```rust,ignore
    /// View::image()
    ///     .data(include_bytes!("logo.png"))
    ///     .build()
    /// ```
    pub fn data(mut self, bytes: &[u8]) -> Self {
        self.source = Some(crate::image::ImageSource::Data(bytes.to_vec()));

        // Try to detect dimensions for layout
        if let Some((w, h)) = crate::image::detect_image_dimensions(bytes) {
            let (cw, ch) = crate::image::pixels_to_cells(w, h);
            if self.cell_width.is_none() {
                self.cell_width = Some(cw);
            }
            if self.cell_height.is_none() {
                self.cell_height = Some(ch);
            }
        }

        self
    }

    /// Set the image source from a file path.
    ///
    /// The file is loaded at render time.
    ///
    /// # Example
    /// ```rust,ignore
    /// View::image()
    ///     .file("assets/animation.gif")
    ///     .build()
    /// ```
    pub fn file(mut self, path: impl Into<String>) -> Self {
        self.source = Some(crate::image::ImageSource::File(path.into()));
        self
    }

    /// Set explicit width in terminal cells.
    pub fn width(mut self, cells: u16) -> Self {
        self.cell_width = Some(cells);
        self
    }

    /// Set explicit height in terminal cells.
    pub fn height(mut self, cells: u16) -> Self {
        self.cell_height = Some(cells);
        self
    }

    /// Set a specific image ID (for manual caching control).
    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    /// Set alt text for accessibility or fallback display.
    pub fn alt(mut self, text: impl Into<String>) -> Self {
        self.alt = Some(text.into());
        self
    }

    pub fn build(self) -> View {
        View::Image(ImageNode {
            source: self.source,
            id: self.id,
            cell_width: self.cell_width,
            cell_height: self.cell_height,
            alt: self.alt,
        })
    }
}

/// Node representing an interactive PTY terminal emulator.
///
/// **Experimental Preview** - See `View::terminal()` for limitations.
#[derive(Clone)]
pub struct TerminalNode {
    /// Handle to the running PTY process.
    pub handle: crate::terminal_state::TerminalHandle,
    /// Visible rows (defaults to 24).
    pub rows: usize,
    /// Visible columns (defaults to 80).
    pub cols: usize,
    /// Show border around terminal.
    pub border: bool,
    /// Title displayed in border (if border is enabled).
    pub title: Option<String>,
    /// Callback invoked when the PTY process exits.
    pub on_exit: Option<Callback>,
}

/// Builder for Terminal views.
pub struct TerminalBuilder {
    handle: Option<crate::terminal_state::TerminalHandle>,
    rows: usize,
    cols: usize,
    border: bool,
    title: Option<String>,
    on_exit: Option<Callback>,
}

impl Default for TerminalBuilder {
    fn default() -> Self {
        Self {
            handle: None,
            rows: 24,
            cols: 80,
            border: true,
            title: Some("Terminal".to_string()),
            on_exit: None,
        }
    }
}

impl TerminalBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the terminal handle (required).
    ///
    /// Get a handle from `cx.use_terminal()` in your component.
    pub fn handle(mut self, handle: crate::terminal_state::TerminalHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Set the number of visible rows (default: 24).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    /// Set the number of visible columns (default: 80).
    pub fn cols(mut self, cols: usize) -> Self {
        self.cols = cols;
        self
    }

    /// Enable or disable border (default: true).
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    /// Set the border title (default: "Terminal").
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set a callback to be invoked when the PTY process exits.
    pub fn on_exit(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_exit = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> View {
        View::Terminal(TerminalNode {
            handle: self.handle.expect("Terminal requires a handle (from cx.use_terminal())"),
            rows: self.rows,
            cols: self.cols,
            border: self.border,
            title: self.title,
            on_exit: self.on_exit,
        })
    }
}

// ========== Error Boundary ==========

/// An error boundary that catches panics in its child view.
///
/// During rendering, if the child view panics, the fallback view is
/// displayed instead. This prevents a single misbehaving component
/// from crashing the entire application.
#[derive(Clone)]
pub struct ErrorBoundaryNode {
    /// The child view to render (may panic).
    pub child: Box<View>,
    /// The fallback view shown when the child panics.
    pub fallback: Box<View>,
}

/// Builder for error boundary views.
#[derive(Default)]
pub struct ErrorBoundaryBuilder {
    child: Option<View>,
    fallback: Option<View>,
}

impl ErrorBoundaryBuilder {
    pub fn new() -> Self {
        Self {
            child: None,
            fallback: None,
        }
    }

    /// Set the child view (the view that might panic).
    pub fn child(mut self, child: View) -> Self {
        self.child = Some(child);
        self
    }

    /// Set the fallback view (shown when child panics).
    pub fn fallback(mut self, fallback: View) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn build(self) -> View {
        View::ErrorBoundary(ErrorBoundaryNode {
            child: Box::new(self.child.unwrap_or(View::Empty)),
            fallback: Box::new(
                self.fallback
                    .unwrap_or_else(|| View::text("[error boundary: child panicked]")),
            ),
        })
    }
}

// ========== Custom Widget ==========

/// A node wrapping a user-defined custom widget.
///
/// Uses `Rc<RefCell<dyn Widget>>` because:
/// - `Rc` enables Clone (View derives Clone) without requiring Widget: Clone
/// - `RefCell` allows interior mutability for focus handling
#[derive(Clone)]
pub struct CustomNode {
    pub widget: Rc<RefCell<dyn Widget>>,
}

// ========== Slider ==========

/// A slider for bounded numeric values (e.g., MIDI CC, volume, brightness).
#[derive(Clone)]
pub struct SliderNode {
    pub min: f64,
    pub max: f64,
    pub value: f64,
    pub step: f64,
    pub label: Option<String>,
    pub on_change: Option<SliderCallback>,
    pub color: Option<crossterm::style::Color>,
}

/// Builder for slider views.
#[derive(Default)]
pub struct SliderBuilder {
    min: f64,
    max: f64,
    value: f64,
    step: f64,
    label: Option<String>,
    on_change: Option<SliderCallback>,
    color: Option<crossterm::style::Color>,
}

impl SliderBuilder {
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            value: 0.0,
            step: 1.0,
            label: None,
            on_change: None,
            color: None,
        }
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_change(mut self, callback: impl Fn(f64) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    pub fn color(mut self, color: crossterm::style::Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn build(self) -> View {
        View::Slider(SliderNode {
            min: self.min,
            max: self.max,
            value: self.value.clamp(self.min, self.max),
            step: self.step,
            label: self.label,
            on_change: self.on_change,
            color: self.color,
        })
    }
}
