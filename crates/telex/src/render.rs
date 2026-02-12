use std::panic::AssertUnwindSafe;

use crate::buffer::{Buffer, Rect};
use crate::canvas::PendingCanvas;
use crate::image::PendingImage;
use crate::text;
use crate::theme::current_theme;
use crate::view::{
    BoxNode, ButtonNode, CanvasNode, CheckboxNode, ColumnWidth, CommandPaletteNode,
    ErrorBoundaryNode, FormFieldNode, FormNode, HStackNode, ImageNode, ListNode, MenuBarNode,
    MenuItemNode, ModalNode, Orientation, PaletteCommand, ProgressBarNode, RadioGroupNode,
    SliderNode, SplitNode, StatusBarNode, TabPosition, TableNode, TabsNode, TerminalNode,
    TextAlign, TextAreaNode, TextInputNode, TextNode, ToastContainerNode, ToastLevelView,
    ToastPosition, TreeItem, TreeNode, TreePath, VStackNode, View,
};

/// A pending menu dropdown to render after main content.
struct PendingDropdown {
    menu: crate::view::Menu,
    x: u16,
    y: u16,
    selected: usize,
}

/// Context passed during rendering to track focus and scroll state.
pub struct RenderContext {
    /// Index of the currently focused element
    pub focus_index: usize,
    /// Counter for focusable elements encountered during render
    focusable_counter: usize,
    /// Scroll offsets for each focusable (scroll_y, scroll_x)
    scroll_offsets: Vec<(u16, u16)>,
    /// Cursor positions for text inputs (usize::MAX means use node's value)
    cursor_offsets: Vec<usize>,
    /// Root area (full screen) for modals to use
    pub root_area: Rect,
    /// Whether a modal is visible (focusables outside modal should be skipped)
    modal_visible: bool,
    /// Whether we're currently rendering inside a modal's content
    inside_modal: bool,
    /// Whether focus styling should be shown (false until user starts keyboard navigation)
    focus_visible: bool,
    /// Pending menu dropdowns to render after main content (as overlays)
    pending_dropdowns: Vec<PendingDropdown>,
    /// Pending canvas graphics to render via Kitty protocol (after character buffer)
    pending_canvases: Vec<PendingCanvas>,
    /// Pending images to render via Kitty protocol (after character buffer)
    pending_images: Vec<PendingImage>,
}

impl RenderContext {
    pub fn new(
        focus_index: usize,
        focus_visible: bool,
        scroll_offsets: Vec<(u16, u16)>,
        cursor_offsets: Vec<usize>,
        root_area: Rect,
    ) -> Self {
        Self {
            focus_index,
            focusable_counter: 0,
            scroll_offsets,
            cursor_offsets,
            root_area,
            modal_visible: false,
            inside_modal: false,
            focus_visible,
            pending_dropdowns: Vec::new(),
            pending_canvases: Vec::new(),
            pending_images: Vec::new(),
        }
    }

    /// Get cursor position for a focusable at the given index.
    /// Returns None if not set (should use node's value).
    fn cursor_offset(&self, index: usize) -> Option<usize> {
        self.cursor_offsets.get(index).filter(|&&p| p != usize::MAX).copied()
    }

    /// Set whether a modal is visible (call before rendering starts).
    pub fn set_modal_visible(&mut self, visible: bool) {
        self.modal_visible = visible;
    }

    /// Queue a menu dropdown to be rendered as an overlay after main content.
    fn queue_dropdown(&mut self, menu: crate::view::Menu, x: u16, y: u16, selected: usize) {
        self.pending_dropdowns.push(PendingDropdown {
            menu,
            x,
            y,
            selected,
        });
    }

    /// Render all pending dropdowns (call after main render pass).
    pub fn render_pending_dropdowns(&mut self, buffer: &mut Buffer) {
        // Take ownership of dropdowns to avoid borrow issues
        let dropdowns: Vec<_> = self.pending_dropdowns.drain(..).collect();
        for dropdown in dropdowns {
            render_menu_dropdown_impl(
                buffer,
                &dropdown.menu,
                dropdown.x,
                dropdown.y,
                dropdown.selected,
                self,
            );
        }
    }

    /// Check if the next focusable element is focused, and increment counter.
    /// Returns false if focus styling is not visible yet (user hasn't started navigating).
    /// When a modal is visible but we're not inside it, skip counting.
    fn is_next_focused(&mut self) -> bool {
        // Skip focus counting for elements outside modal when modal is visible
        if self.modal_visible && !self.inside_modal {
            return false;
        }
        let is_focused = self.focus_visible && self.focusable_counter == self.focus_index;
        self.focusable_counter += 1;
        is_focused
    }

    /// Check if the next focusable element is focused, ignoring focus_visible.
    /// Used for text input cursors which must show even before Tab is pressed.
    /// IMPORTANT: Call this INSTEAD of is_next_focused, not in addition to it.
    fn is_next_focused_for_cursor(&mut self) -> bool {
        if self.modal_visible && !self.inside_modal {
            return false;
        }
        let is_focused = self.focusable_counter == self.focus_index;
        self.focusable_counter += 1;
        is_focused
    }

    /// Get the current focusable index without advancing.
    fn current_focusable_index(&self) -> usize {
        self.focusable_counter
    }

    /// Clamp scroll offset for a focusable to a maximum value.
    fn clamp_scroll_y(&mut self, idx: usize, max: u16) {
        if let Some((scroll_y, _)) = self.scroll_offsets.get_mut(idx) {
            if *scroll_y > max {
                *scroll_y = max;
            }
        }
    }

    /// Get the (potentially modified) scroll offsets to apply back to FocusManager.
    pub fn scroll_offsets(&self) -> &[(u16, u16)] {
        &self.scroll_offsets
    }

    /// Queue a canvas graphic to be rendered via Kitty protocol after character buffer.
    pub fn queue_canvas(&mut self, canvas: PendingCanvas) {
        self.pending_canvases.push(canvas);
    }

    /// Take the pending canvases for Kitty protocol rendering.
    pub fn take_pending_canvases(&mut self) -> Vec<PendingCanvas> {
        std::mem::take(&mut self.pending_canvases)
    }

    /// Queue an image to be rendered via Kitty protocol after character buffer.
    pub fn queue_image(&mut self, image: PendingImage) {
        self.pending_images.push(image);
    }

    /// Take the pending images for Kitty protocol rendering.
    pub fn take_pending_images(&mut self) -> Vec<PendingImage> {
        std::mem::take(&mut self.pending_images)
    }
}

/// Render a View tree into a buffer within the given area.
pub fn render_view(buffer: &mut Buffer, view: &View, area: Rect, ctx: &mut RenderContext) {
    match view {
        View::Text(node) => render_text(buffer, node, area),
        View::VStack(node) => render_vstack(buffer, node, area, ctx),
        View::HStack(node) => render_hstack(buffer, node, area, ctx),
        View::Button(node) => render_button(buffer, node, area, ctx),
        View::Box(node) => render_box(buffer, node, area, ctx),
        View::List(node) => render_list(buffer, node, area, ctx),
        View::TextInput(node) => render_text_input(buffer, node, area, ctx),
        View::TextArea(node) => render_text_area(buffer, node, area, ctx),
        View::Checkbox(node) => render_checkbox(buffer, node, area, ctx),
        View::RadioGroup(node) => render_radio_group(buffer, node, area, ctx),
        View::Modal(node) => render_modal(buffer, node, area, ctx),
        View::Split(node) => render_split(buffer, node, area, ctx),
        View::Tabs(node) => render_tabs(buffer, node, area, ctx),
        View::Tree(node) => render_tree(buffer, node, area, ctx),
        View::Table(node) => render_table(buffer, node, area, ctx),
        View::ProgressBar(node) => render_progress_bar(buffer, node, area),
        View::StatusBar(node) => render_status_bar(buffer, node, area),
        View::CommandPalette(node) => render_command_palette(buffer, node, area, ctx),
        View::MenuBar(node) => render_menu_bar(buffer, node, area, ctx),
        View::ToastContainer(node) => render_toast_container(buffer, node, ctx),
        View::Form(node) => render_form(buffer, node, area, ctx),
        View::FormField(node) => render_form_field(buffer, node, area, ctx),
        View::Canvas(node) => render_canvas(buffer, node, area, ctx),
        View::Image(node) => render_image(buffer, node, area, ctx),
        View::Terminal(node) => render_terminal(buffer, node, area, ctx),
        View::ErrorBoundary(node) => render_error_boundary(buffer, node, area, ctx),
        View::Custom(node) => node.widget.borrow().render(area, buffer),
        View::Slider(node) => render_slider(buffer, node, area, ctx),
        View::Spacer(_) => {} // Spacer is handled by parent layout
        View::Empty => {}
    }
}

fn render_text(buffer: &mut Buffer, node: &TextNode, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();
    let fg = node.color.unwrap_or(theme.foreground);
    let bg = node.bg_color.unwrap_or(theme.background);

    let width = area.width as usize;
    let mut row = 0u16;

    // Split text into lines, then wrap each line using grapheme-aware soft wrapping
    for line in node.content.lines() {
        if row >= area.height {
            break;
        }

        // Soft-wrap this line (visual only, never modifies content)
        let wrapped = text::soft_wrap_line(line, width);
        for visual_line in wrapped {
            if row >= area.height {
                break;
            }
            buffer.write_str_styled(
                area.x,
                area.y + row,
                visual_line.text,
                fg,
                bg,
                node.bold,
                node.italic,
                node.underline,
                node.dim,
            );
            row += 1;
        }
    }
}

/// Calculate the height of a view considering text wrapping at the given width.
fn wrapped_height(view: &View, width: u16) -> u16 {
    match view {
        View::Text(n) => {
            let w = width as usize;
            if w == 0 {
                return 1;
            }
            // Count wrapped lines using grapheme-aware soft wrapping
            let height = text::wrapped_height(&n.content, w);
            if height == 0 && n.content.is_empty() {
                1
            } else {
                (height as u16).max(1)
            }
        }
        View::VStack(n) => {
            let spacing = if n.children.len() > 1 {
                n.spacing * (n.children.len() as u16 - 1)
            } else {
                0
            };
            let children_height: u16 = n.children.iter().map(|c| wrapped_height(c, width)).sum();
            children_height + spacing
        }
        View::HStack(n) => {
            if n.children.is_empty() {
                return 1;
            }
            // Calculate widths similar to render_hstack: use intrinsic widths
            let spacing = if n.children.len() > 1 {
                n.spacing * (n.children.len() as u16 - 1)
            } else {
                0
            };
            let available = width.saturating_sub(spacing);

            // Get intrinsic widths for non-flex children
            let mut total_fixed: u16 = 0;
            let mut flex_count = 0;
            for child in &n.children {
                if child.flex() > 0 {
                    flex_count += 1;
                } else {
                    total_fixed += child.intrinsic_width().unwrap_or(1);
                }
            }

            let flex_space = available.saturating_sub(total_fixed);
            let flex_each = if flex_count > 0 {
                flex_space / flex_count as u16
            } else {
                0
            };

            // Calculate max height of children using their actual widths
            n.children
                .iter()
                .map(|c| {
                    let child_width = if c.flex() > 0 {
                        flex_each.max(1)
                    } else {
                        c.intrinsic_width().unwrap_or(1)
                    };
                    wrapped_height(c, child_width)
                })
                .max()
                .unwrap_or(1)
        }
        View::Box(n) => {
            let border = if n.border { 2 } else { 0 };
            let padding = n.padding * 2;
            // For scrollable boxes, content scrolls so don't include full content height
            if n.scroll || n.auto_scroll_bottom {
                border + padding + 1 // Minimal height, content will scroll
            } else {
                let inner_width = width.saturating_sub(border + padding);
                let inner = n
                    .child
                    .as_ref()
                    .map(|c| wrapped_height(c, inner_width))
                    .unwrap_or(0);
                inner + border + padding
            }
        }
        View::ErrorBoundary(n) => wrapped_height(&n.child, width),
        _ => view.intrinsic_height().unwrap_or(1),
    }
}

fn render_error_boundary(
    buffer: &mut Buffer,
    node: &ErrorBoundaryNode,
    area: Rect,
    ctx: &mut RenderContext,
) {
    // Try rendering the child; if it panics, render the fallback instead.
    // Tell the panic hook to stay quiet — we'll handle it here.
    crate::IN_ERROR_BOUNDARY.with(|f| f.set(true));
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        render_view(buffer, &node.child, area, ctx);
    }));
    crate::IN_ERROR_BOUNDARY.with(|f| f.set(false));

    if result.is_err() {
        // Child panicked — clear the area and render fallback
        let theme = current_theme();
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buffer.set(x, y, ' ', theme.foreground, theme.background);
            }
        }
        render_view(buffer, &node.fallback, area, ctx);
    }
}

fn render_slider(buffer: &mut Buffer, node: &SliderNode, area: Rect, ctx: &mut RenderContext) {
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Format: [label] [====>--------] value
    let value_str = if node.step >= 1.0 {
        format!("{}", node.value as i64)
    } else {
        format!("{:.1}", node.value)
    };

    // Show focus marker: "▸ " when focused, "  " when not (keeps alignment stable)
    let focus_marker = if is_focused { "▸ " } else { "  " };

    let label_prefix = match &node.label {
        Some(l) => format!("{}{} ", focus_marker, l),
        None => format!("{}", focus_marker),
    };

    // Reserve space: focus marker + label + value display + brackets + 1 space
    let reserved = label_prefix.len() + value_str.len() + 3; // "[] " + value
    let track_width = if (area.width as usize) > reserved + 4 {
        area.width as usize - reserved
    } else {
        4 // Minimum track width
    };

    // Calculate fill position
    let range = node.max - node.min;
    let ratio = if range > 0.0 {
        ((node.value - node.min) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled = (ratio * track_width as f64).round() as usize;
    let empty = track_width.saturating_sub(filled);

    // Choose colors — use custom color if set, otherwise theme primary/secondary
    let default_fill = if is_focused { theme.primary } else { theme.secondary };
    let fill_fg = node.color.unwrap_or(default_fill);
    let fg = theme.foreground;

    // Write label prefix (bold when focused)
    let label_bold = is_focused;
    let mut x = area.x;
    for ch in label_prefix.chars() {
        if x >= area.x + area.width {
            break;
        }
        buffer.set_cell(x, area.y, crate::buffer::Cell::styled(ch, fg, theme.background, label_bold, false, false, false));
        x += 1;
    }

    // Write '['
    if x < area.x + area.width {
        buffer.set(x, area.y, '[', fg, theme.background);
        x += 1;
    }

    // Write filled portion (━ and ─ are the same box-drawing weight
    // so the track doesn't visually jump when the fill ratio changes)
    for _ in 0..filled {
        if x >= area.x + area.width {
            break;
        }
        buffer.set(x, area.y, '━', fill_fg, theme.background);
        x += 1;
    }

    // Write empty portion
    for _ in 0..empty {
        if x >= area.x + area.width {
            break;
        }
        buffer.set(x, area.y, '─', fg, theme.background);
        x += 1;
    }

    // Write ']'
    if x < area.x + area.width {
        buffer.set(x, area.y, ']', fg, theme.background);
        x += 1;
    }

    // Write space + value
    if x < area.x + area.width {
        buffer.set(x, area.y, ' ', fg, theme.background);
        x += 1;
    }
    for ch in value_str.chars() {
        if x >= area.x + area.width {
            break;
        }
        buffer.set(x, area.y, ch, fg, theme.background);
        x += 1;
    }
}

fn render_button(buffer: &mut Buffer, node: &ButtonNode, area: Rect, ctx: &mut RenderContext) {
    // Always increment focusable counter even if we can't render
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }
    let theme = current_theme();

    // Format button: [ label ] - focus indicated by color only
    let label = format!("[ {} ]", node.label);

    // Use theme colors
    let (fg, bg) = if is_focused {
        (theme.button_focused_fg, theme.button_focused_bg)
    } else {
        (theme.button_fg, theme.button_bg)
    };

    buffer.write_str(area.x, area.y, &label, fg, bg);
}

fn render_vstack(buffer: &mut Buffer, node: &VStackNode, area: Rect, ctx: &mut RenderContext) {
    use crate::view::LayoutMode;

    if node.children.is_empty() || area.height == 0 {
        return;
    }

    // Dispatch to layout algorithm based on mode
    match node.layout_mode {
        LayoutMode::Flex => render_vstack_flex(buffer, node, area, ctx),
    }
}

fn render_vstack_flex(buffer: &mut Buffer, node: &VStackNode, area: Rect, ctx: &mut RenderContext) {
    use crate::view::{Align, Justify};

    let child_count = node.children.len();

    // Calculate total spacing needed
    let base_spacing = if child_count > 1 {
        node.spacing * (child_count as u16 - 1)
    } else {
        0
    };

    // Available height after spacing
    let available_height = area.height.saturating_sub(base_spacing);

    // First pass: calculate child heights and flex totals
    let mut total_flex: u16 = 0;
    let mut total_min_height: u16 = 0;
    let mut child_heights: Vec<u16> = Vec::with_capacity(child_count);

    for child in &node.children {
        let flex = child.flex();
        // Use width-aware height calculation for text wrapping
        let min_h = child
            .min_height()
            .unwrap_or_else(|| wrapped_height(child, area.width));
        child_heights.push(min_h);
        total_min_height += min_h;

        if flex > 0 {
            total_flex += flex;
        }
    }

    // Space available for flex distribution
    let flex_space = available_height.saturating_sub(total_min_height);

    // Second pass: calculate final heights (applying flex)
    for (i, child) in node.children.iter().enumerate() {
        let flex = child.flex();
        let max_h = child.max_height();

        if flex > 0 && total_flex > 0 {
            let flex_share = (flex_space * flex) / total_flex;
            child_heights[i] = child_heights[i].saturating_add(flex_share);
        }

        // Apply max constraint
        if let Some(max) = max_h {
            child_heights[i] = child_heights[i].min(max);
        }
    }

    // Calculate total content height
    let total_content_height: u16 = child_heights.iter().sum::<u16>() + base_spacing;
    let remaining_space = area.height.saturating_sub(total_content_height);

    // Calculate starting position and spacing based on justify
    let (start_y, extra_spacing) = match node.justify {
        Justify::Start => (area.y, 0u16),
        Justify::End => (area.y + remaining_space, 0),
        Justify::Center => (area.y + remaining_space / 2, 0),
        Justify::SpaceBetween => {
            if child_count > 1 {
                (area.y, remaining_space / (child_count as u16 - 1))
            } else {
                (area.y, 0)
            }
        }
        Justify::SpaceAround => {
            let gap = remaining_space / (child_count as u16 + 1);
            (area.y + gap, gap)
        }
    };

    // Third pass: render children
    let mut y = start_y;
    for (i, child) in node.children.iter().enumerate() {
        let h = child_heights[i];

        if h > 0 && y < area.y + area.height {
            let actual_h = h.min(area.y + area.height - y);

            // Calculate x and width based on align (cross axis)
            let (child_x, child_w) = match node.align {
                Align::Stretch => (area.x, area.width),
                Align::Start => {
                    let w = child
                        .intrinsic_width()
                        .unwrap_or(area.width)
                        .min(area.width);
                    (area.x, w)
                }
                Align::End => {
                    let w = child
                        .intrinsic_width()
                        .unwrap_or(area.width)
                        .min(area.width);
                    (area.x + area.width - w, w)
                }
                Align::Center => {
                    let w = child
                        .intrinsic_width()
                        .unwrap_or(area.width)
                        .min(area.width);
                    (area.x + (area.width - w) / 2, w)
                }
            };

            let child_area = Rect::new(child_x, y, child_w, actual_h);
            render_view(buffer, child, child_area, ctx);
        }

        y += h;

        // Add spacing after all but last child
        if i < child_count - 1 {
            y += node.spacing + extra_spacing;
        }
    }
}

fn render_hstack(buffer: &mut Buffer, node: &HStackNode, area: Rect, ctx: &mut RenderContext) {
    use crate::view::LayoutMode;

    if node.children.is_empty() || area.width == 0 {
        return;
    }

    // Dispatch to layout algorithm based on mode
    match node.layout_mode {
        LayoutMode::Flex => render_hstack_flex(buffer, node, area, ctx),
    }
}

fn render_hstack_flex(buffer: &mut Buffer, node: &HStackNode, area: Rect, ctx: &mut RenderContext) {
    use crate::view::{Align, Justify};

    let child_count = node.children.len();

    // Calculate total spacing needed
    let base_spacing = if child_count > 1 {
        node.spacing * (child_count as u16 - 1)
    } else {
        0
    };

    // Available width after spacing
    let available_width = area.width.saturating_sub(base_spacing);

    // First pass: calculate child widths and flex totals
    let mut total_flex: u16 = 0;
    let mut total_min_width: u16 = 0;
    let mut child_widths: Vec<u16> = Vec::with_capacity(child_count);

    for child in &node.children {
        let flex = child.flex();
        let min_w = child
            .min_width()
            .or_else(|| child.intrinsic_width())
            .unwrap_or(1);
        child_widths.push(min_w);
        total_min_width += min_w;

        if flex > 0 {
            total_flex += flex;
        }
    }

    // Space available for flex distribution
    let flex_space = available_width.saturating_sub(total_min_width);

    // Second pass: calculate final widths (applying flex)
    for (i, child) in node.children.iter().enumerate() {
        let flex = child.flex();
        let max_w = child.max_width();

        if flex > 0 && total_flex > 0 {
            let flex_share = (flex_space * flex) / total_flex;
            child_widths[i] = child_widths[i].saturating_add(flex_share);
        }

        // Apply max constraint
        if let Some(max) = max_w {
            child_widths[i] = child_widths[i].min(max);
        }
    }

    // Calculate total content width
    let total_content_width: u16 = child_widths.iter().sum::<u16>() + base_spacing;
    let remaining_space = area.width.saturating_sub(total_content_width);

    // Calculate starting position and spacing based on justify
    let (start_x, extra_spacing) = match node.justify {
        Justify::Start => (area.x, 0u16),
        Justify::End => (area.x + remaining_space, 0),
        Justify::Center => (area.x + remaining_space / 2, 0),
        Justify::SpaceBetween => {
            if child_count > 1 {
                (area.x, remaining_space / (child_count as u16 - 1))
            } else {
                (area.x, 0)
            }
        }
        Justify::SpaceAround => {
            let gap = remaining_space / (child_count as u16 + 1);
            (area.x + gap, gap)
        }
    };

    // Third pass: render children
    let mut x = start_x;
    for (i, child) in node.children.iter().enumerate() {
        let w = child_widths[i];

        if w > 0 && x < area.x + area.width {
            let actual_w = w.min(area.x + area.width - x);

            // Calculate y and height based on align (cross axis)
            // Use wrapped_height to account for text wrapping at the given width
            let (child_y, child_h) = match node.align {
                Align::Stretch => (area.y, area.height),
                Align::Start => {
                    let h = wrapped_height(child, actual_w).min(area.height);
                    (area.y, h)
                }
                Align::End => {
                    let h = wrapped_height(child, actual_w).min(area.height);
                    (area.y + area.height - h, h)
                }
                Align::Center => {
                    let h = wrapped_height(child, actual_w).min(area.height);
                    (area.y + (area.height - h) / 2, h)
                }
            };

            let child_area = Rect::new(x, child_y, actual_w, child_h);
            render_view(buffer, child, child_area, ctx);
        }

        x += w;

        // Add spacing after all but last child
        if i < child_count - 1 {
            x += node.spacing + extra_spacing;
        }
    }
}

fn render_box(buffer: &mut Buffer, node: &BoxNode, area: Rect, ctx: &mut RenderContext) {
    let is_scrollable = node.scroll || node.auto_scroll_bottom;
    let is_focusable = node.focusable;
    let _ = is_scrollable; // Used implicitly for scroll behavior

    if area.width == 0 || area.height == 0 {
        // Still need to count focusables in children even if we can't render
        if is_focusable {
            ctx.focusable_counter += 1;
        }
        if let Some(child) = &node.child {
            count_focusables(child, ctx);
        }
        return;
    }

    // Get scroll offset and focus state if focusable
    let (user_scroll_y, is_focused) = if is_focusable {
        let idx = ctx.current_focusable_index();
        let is_focused = ctx.is_next_focused();
        let (sy, _sx) = ctx.scroll_offsets.get(idx).copied().unwrap_or((0, 0));
        (sy, is_focused)
    } else {
        (0, false)
    };

    // Draw border if enabled (with focus highlight when focused)
    if node.border {
        if is_focusable && is_focused {
            draw_border_focused(buffer, area);
        } else {
            draw_border(buffer, area);
        }
    }

    // Calculate inner area (accounting for border and padding)
    let border_offset = if node.border { 1 } else { 0 };
    let total_offset = border_offset + node.padding;

    if area.width <= total_offset * 2 || area.height <= total_offset * 2 {
        // Still need to count focusables in children even if we can't render
        if let Some(child) = &node.child {
            count_focusables(child, ctx);
        }
        return;
    }

    let inner_area = Rect::new(
        area.x + total_offset,
        area.y + total_offset,
        area.width - total_offset * 2,
        area.height - total_offset * 2,
    );

    // Render child if present
    if let Some(child) = &node.child {
        // Reserve space for scrollbar if content might overflow
        let content_height = wrapped_height(child, inner_area.width);
        let needs_scrollbar = content_height > inner_area.height;

        // Reduce inner area width by 1 if we need a scrollbar
        let content_area = if needs_scrollbar && inner_area.width > 1 {
            Rect::new(
                inner_area.x,
                inner_area.y,
                inner_area.width - 1,
                inner_area.height,
            )
        } else {
            inner_area
        };

        if node.auto_scroll_bottom && content_height > content_area.height {
            // Auto-scroll mode: default to bottom, but allow user to scroll up
            // user_scroll_y represents offset FROM bottom (0 = at bottom, max = at top)
            let max_scroll = content_height - content_area.height;
            // Clamp scroll offset to actual max (prevents overscroll accumulation)
            let scroll_idx = ctx.current_focusable_index().saturating_sub(1);
            ctx.clamp_scroll_y(scroll_idx, max_scroll);
            let clamped_scroll_y = ctx
                .scroll_offsets
                .get(scroll_idx)
                .map(|(y, _)| *y)
                .unwrap_or(0);
            let effective_scroll = max_scroll.saturating_sub(clamped_scroll_y);
            render_scrolled(buffer, child, content_area, effective_scroll, ctx);
            draw_scrollbar(buffer, inner_area, content_height, effective_scroll);
        } else if node.scroll && needs_scrollbar {
            // Manual scroll mode
            let max_scroll = content_height.saturating_sub(content_area.height);
            let effective_scroll = user_scroll_y.min(max_scroll);
            render_scrolled(buffer, child, content_area, effective_scroll, ctx);
            draw_scrollbar(buffer, inner_area, content_height, effective_scroll);
        } else {
            // Content fits, render normally
            render_view(buffer, child, content_area, ctx);
        }
    }
}

/// Draw a scrollbar on the right edge of the area.
fn draw_scrollbar(buffer: &mut Buffer, area: Rect, content_height: u16, scroll_y: u16) {
    if area.height == 0 || content_height == 0 {
        return;
    }

    let theme = current_theme();
    let scrollbar_x = area.x + area.width - 1;
    let track_height = area.height as f32;
    let content_h = content_height as f32;
    let visible_h = area.height as f32;

    // Calculate thumb size (proportional to visible/total content)
    let thumb_size = ((visible_h / content_h) * track_height).max(1.0) as u16;
    let thumb_size = thumb_size.min(area.height);

    // Calculate thumb position
    let max_scroll = content_height.saturating_sub(area.height);
    let scroll_ratio = if max_scroll > 0 {
        scroll_y as f32 / max_scroll as f32
    } else {
        0.0
    };
    let thumb_travel = area.height.saturating_sub(thumb_size);
    let thumb_y = area.y + (scroll_ratio * thumb_travel as f32) as u16;

    // Draw track and thumb
    for y in area.y..area.y + area.height {
        let is_thumb = y >= thumb_y && y < thumb_y + thumb_size;
        let (ch, fg) = if is_thumb {
            ('┃', theme.foreground)
        } else {
            ('│', theme.muted)
        };
        buffer.set(scrollbar_x, y, ch, fg, theme.background);
    }
}

/// Render a view with vertical scrolling applied.
fn render_scrolled(
    buffer: &mut Buffer,
    view: &View,
    area: Rect,
    scroll_y: u16,
    ctx: &mut RenderContext,
) {
    // For scrolling, we need to know the content height
    // For now, we'll handle VStack specially (most common case for scrollable content)
    match view {
        View::VStack(node) => {
            render_vstack_scrolled(buffer, node, area, scroll_y, ctx);
        }
        View::Text(node) => {
            // For multi-line text, skip scroll_y lines
            let theme = current_theme();
            let fg = node.color.unwrap_or(theme.foreground);
            let bg = node.bg_color.unwrap_or(theme.background);
            let lines: Vec<&str> = node.content.lines().collect();
            for (i, line) in lines.iter().skip(scroll_y as usize).enumerate() {
                if i as u16 >= area.height {
                    break;
                }
                buffer.write_str_styled(
                    area.x,
                    area.y + i as u16,
                    line,
                    fg,
                    bg,
                    node.bold,
                    node.italic,
                    node.underline,
                    node.dim,
                );
            }
        }
        _ => {
            // For other views, just render normally (scroll doesn't apply well)
            render_view(buffer, view, area, ctx);
        }
    }
}

/// Render a VStack with scrolling.
fn render_vstack_scrolled(
    buffer: &mut Buffer,
    node: &VStackNode,
    area: Rect,
    scroll_y: u16,
    ctx: &mut RenderContext,
) {
    if node.children.is_empty() || area.height == 0 {
        return;
    }

    // Calculate heights for each child using wrapped_height
    let child_heights: Vec<u16> = node
        .children
        .iter()
        .map(|c| wrapped_height(c, area.width))
        .collect();

    // Track cumulative y position in content space
    let mut content_y: u16 = 0;

    for (i, child) in node.children.iter().enumerate() {
        let child_height = child_heights[i];
        let child_end_y = content_y + child_height;

        // Skip children completely above the scroll window
        if child_end_y <= scroll_y {
            count_focusables(child, ctx);
            content_y = child_end_y + node.spacing;
            continue;
        }

        // Stop if we're completely past the visible area
        if content_y >= scroll_y + area.height {
            count_focusables(child, ctx);
            content_y = child_end_y + node.spacing;
            continue;
        }

        // Check if this child starts above the scroll window (partially scrolled)
        if content_y < scroll_y {
            // This child is partially scrolled - need to skip some of its content
            let skip_lines = scroll_y - content_y;
            let visible_height = child_height.saturating_sub(skip_lines).min(area.height);

            if visible_height > 0 {
                let child_area = Rect::new(area.x, area.y, area.width, visible_height);
                // Render with internal scroll offset
                render_scrolled(buffer, child, child_area, skip_lines, ctx);
            } else {
                count_focusables(child, ctx);
            }
        } else {
            // This child starts at or below the scroll window - render from its top
            let visible_y = area.y + content_y - scroll_y;
            let remaining_height = (area.y + area.height).saturating_sub(visible_y);
            let render_height = child_height.min(remaining_height);

            if render_height > 0 {
                let child_area = Rect::new(area.x, visible_y, area.width, render_height);
                render_view(buffer, child, child_area, ctx);
            } else {
                count_focusables(child, ctx);
            }
        }

        content_y = child_end_y + node.spacing;
    }
}

/// Count focusables in a view without rendering (to keep counter in sync).
fn count_focusables(view: &View, ctx: &mut RenderContext) {
    match view {
        View::Button(_)
        | View::List(_)
        | View::TextInput(_)
        | View::TextArea(_)
        | View::Checkbox(_)
        | View::Tree(_)
        | View::Table(_) => {
            ctx.focusable_counter += 1;
        }
        View::Box(node) => {
            if node.focusable {
                ctx.focusable_counter += 1;
            }
            if let Some(child) = &node.child {
                count_focusables(child, ctx);
            }
        }
        View::VStack(node) => {
            for child in &node.children {
                count_focusables(child, ctx);
            }
        }
        View::HStack(node) => {
            for child in &node.children {
                count_focusables(child, ctx);
            }
        }
        View::Modal(node) => {
            if node.visible {
                if let Some(child) = &node.child {
                    count_focusables(child, ctx);
                }
            }
        }
        _ => {}
    }
}

/// Draw a border around the given area using box-drawing characters.
fn draw_border(buffer: &mut Buffer, area: Rect) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let theme = current_theme();
    let fg = theme.border;
    let bg = theme.background;

    // Corners
    buffer.set(area.x, area.y, '┌', fg, bg);
    buffer.set(area.x + area.width - 1, area.y, '┐', fg, bg);
    buffer.set(area.x, area.y + area.height - 1, '└', fg, bg);
    buffer.set(
        area.x + area.width - 1,
        area.y + area.height - 1,
        '┘',
        fg,
        bg,
    );

    // Top and bottom edges
    for x in (area.x + 1)..(area.x + area.width - 1) {
        buffer.set(x, area.y, '─', fg, bg);
        buffer.set(x, area.y + area.height - 1, '─', fg, bg);
    }

    // Left and right edges
    for y in (area.y + 1)..(area.y + area.height - 1) {
        buffer.set(area.x, y, '│', fg, bg);
        buffer.set(area.x + area.width - 1, y, '│', fg, bg);
    }
}

/// Draw a highlighted border for focused scrollable boxes.
fn draw_border_focused(buffer: &mut Buffer, area: Rect) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let theme = current_theme();
    let fg = theme.border_focused;
    let bg = theme.background;

    // Corners (double-line style for focus)
    buffer.set(area.x, area.y, '╔', fg, bg);
    buffer.set(area.x + area.width - 1, area.y, '╗', fg, bg);
    buffer.set(area.x, area.y + area.height - 1, '╚', fg, bg);
    buffer.set(
        area.x + area.width - 1,
        area.y + area.height - 1,
        '╝',
        fg,
        bg,
    );

    // Top and bottom edges
    for x in (area.x + 1)..(area.x + area.width - 1) {
        buffer.set(x, area.y, '═', fg, bg);
        buffer.set(x, area.y + area.height - 1, '═', fg, bg);
    }

    // Left and right edges
    for y in (area.y + 1)..(area.y + area.height - 1) {
        buffer.set(area.x, y, '║', fg, bg);
        buffer.set(area.x + area.width - 1, y, '║', fg, bg);
    }
}

fn render_list(buffer: &mut Buffer, node: &ListNode, area: Rect, ctx: &mut RenderContext) {
    // Always increment focusable counter even if we can't render
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }
    let theme = current_theme();

    let visible_rows = area.height as usize;
    let total_items = node.items.len();

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if node.selected >= visible_rows {
        node.selected - visible_rows + 1
    } else {
        0
    };

    // Determine if we need a scroll indicator
    let needs_scroll_indicator = total_items > visible_rows;
    let content_width = if needs_scroll_indicator {
        area.width.saturating_sub(1) as usize // Reserve 1 char for scroll indicator
    } else {
        area.width as usize
    };

    for row in 0..visible_rows {
        let item_idx = scroll_offset + row;
        if item_idx >= total_items {
            break;
        }

        let item = &node.items[item_idx];
        let is_selected = item_idx == node.selected;

        // Format: > item (selected) or   item (not selected)
        let prefix = if is_selected { "> " } else { "  " };
        let display = format!("{}{}", prefix, item);

        // Truncate if needed
        let display: String = display.chars().take(content_width).collect();

        let (fg, bg) = if is_selected && is_focused {
            (theme.selection_fg, theme.selection_bg)
        } else if is_selected {
            (theme.foreground, theme.muted)
        } else {
            (theme.foreground, theme.background)
        };

        buffer.write_str(area.x, area.y + row as u16, &display, fg, bg);
    }

    // Draw scroll indicator if needed
    if needs_scroll_indicator {
        let scrollbar_x = area.x + area.width - 1;
        let max_scroll = total_items.saturating_sub(visible_rows);

        for row in 0..visible_rows {
            // Calculate which part of the scrollbar this row represents
            let scrollbar_pos = if max_scroll > 0 {
                (scroll_offset * visible_rows) / total_items
            } else {
                0
            };
            let scrollbar_height = (visible_rows * visible_rows) / total_items.max(1);
            let scrollbar_height = scrollbar_height.max(1); // At least 1 char

            let ch = if row >= scrollbar_pos && row < scrollbar_pos + scrollbar_height {
                '█'
            } else {
                '░'
            };
            buffer.set(
                scrollbar_x,
                area.y + row as u16,
                ch,
                theme.muted,
                theme.background,
            );
        }
    }
}

fn render_text_input(
    buffer: &mut Buffer,
    node: &TextInputNode,
    area: Rect,
    ctx: &mut RenderContext,
) {
    // Get focusable index before incrementing counter
    let focusable_idx = ctx.current_focusable_index();
    // Use is_next_focused_for_cursor: text inputs must show cursor even before Tab is pressed
    let is_focused = ctx.is_next_focused_for_cursor();

    if area.width == 0 || area.height == 0 {
        return;
    }
    let theme = current_theme();

    let input_width = area.width as usize;

    // Calculate visible portion based on cursor position
    // Use cursor from context (FocusManager state) if available, otherwise node's value
    let cursor_pos = ctx.cursor_offset(focusable_idx)
        .unwrap_or(node.cursor_pos)
        .min(node.value.len());
    let (visible_text, cursor_offset) = if node.value.is_empty() {
        (String::new(), 0)
    } else {
        // Ensure cursor is visible by scrolling the view
        let text = &node.value;
        if text.len() <= input_width {
            // Text fits, show all
            (text.clone(), cursor_pos)
        } else if cursor_pos < input_width {
            // Cursor near start, show from beginning
            (text[..input_width].to_string(), cursor_pos)
        } else {
            // Scroll to keep cursor visible (cursor at end of visible area)
            let start = cursor_pos.saturating_sub(input_width.saturating_sub(1));
            let end = (start + input_width).min(text.len());
            (text[start..end].to_string(), cursor_pos - start)
        }
    };

    let (fg, bg) = if is_focused {
        (theme.selection_fg, theme.selection_bg)
    } else if node.value.is_empty() {
        (theme.placeholder, theme.input_bg)
    } else {
        (theme.input_fg, theme.input_bg)
    };

    // Build the display string
    //
    // Cursor scenarios:
    //   - end:         cursor past all text         → '█' with cursor/cursor_text
    //   - on_space:    cursor on a space char       → '█' with cursor/cursor_text
    //   - on_char:     cursor on a printable char   → char with cursor/cursor_text
    //   - placeholder: input empty, placeholder shown → first char inverted (bg/placeholder)
    //
    if node.value.is_empty() {
        if is_focused {
            // [placeholder] Focused and empty: show placeholder with first char inverted as cursor
            let padding = input_width.saturating_sub(node.placeholder.len());
            let content = format!("{}{}", node.placeholder, " ".repeat(padding));
            // Fill with placeholder text (dimmed)
            buffer.write_str(area.x, area.y, &content, theme.placeholder, bg);
            // Invert first character to act as cursor
            if !node.placeholder.is_empty() {
                let first_char = node.placeholder.chars().next().unwrap();
                buffer.set_cell(area.x, area.y, crate::buffer::Cell::new(first_char, bg, theme.placeholder));
            } else {
                // [end] No placeholder text - show standard block cursor
                buffer.set_cell(area.x, area.y, crate::buffer::Cell::new('█', theme.cursor, theme.cursor_text));
            }
        } else {
            // Unfocused and empty: just show placeholder
            let padding = input_width.saturating_sub(node.placeholder.len());
            let content = format!("{}{}", node.placeholder, " ".repeat(padding));
            buffer.write_str(area.x, area.y, &content, fg, bg);
        }
    } else {
        // Has content
        let padding = input_width.saturating_sub(visible_text.len());
        let content = format!("{}{}", visible_text, " ".repeat(padding));
        buffer.write_str(area.x, area.y, &content, fg, bg);
        // Draw cursor when focused
        if is_focused {
            let cursor_x = area.x + cursor_offset as u16;
            if cursor_x < area.x + area.width {
                let cursor_char = visible_text.chars().nth(cursor_offset).unwrap_or(' ');
                if cursor_char == ' ' {
                    // [end] or [on_space]: show block cursor
                    buffer.set_cell(cursor_x, area.y, crate::buffer::Cell::new('█', theme.cursor, theme.cursor_text));
                } else {
                    // [on_char]: dark char on bright cursor block (colors swapped)
                    buffer.set_cell(cursor_x, area.y, crate::buffer::Cell::new(cursor_char, theme.cursor_text, theme.cursor));
                }
            }
        }
    }
}

fn render_text_area(buffer: &mut Buffer, node: &TextAreaNode, area: Rect, ctx: &mut RenderContext) {
    // Use is_next_focused_for_cursor: text areas must show cursor even before Tab is pressed
    let is_focused = ctx.is_next_focused_for_cursor();

    if area.width == 0 || area.height == 0 {
        return;
    }
    let theme = current_theme();
    let content_width = area.width.saturating_sub(2) as usize; // Account for border chars

    // Build visual lines by soft-wrapping each logical line
    // Track which visual line corresponds to cursor position
    struct VisualLineInfo<'a> {
        text: &'a str,
        logical_line: usize,
        is_cursor_line: bool,
        cursor_col_in_visual: Option<usize>, // If cursor is on this visual line
    }

    let mut visual_lines: Vec<VisualLineInfo> = Vec::new();
    let mut cursor_visual_row: Option<usize> = None;

    // Cursor scenarios:
    //   - end:         cursor past all text         → '█' with cursor/cursor_text
    //   - on_space:    cursor on a space char       → '█' with cursor/cursor_text
    //   - on_char:     cursor on a printable char   → char with cursor/cursor_text
    //   - placeholder: input empty, placeholder shown → (TextArea shows cursor only, no placeholder inversion)
    //
    if node.value.is_empty() {
        if is_focused {
            // [placeholder] When focused and empty, don't show placeholder - just cursor
            visual_lines.push(VisualLineInfo {
                text: "",
                logical_line: 0,
                is_cursor_line: true,
                cursor_col_in_visual: Some(0),
            });
        } else {
            // When unfocused and empty, show placeholder
            visual_lines.push(VisualLineInfo {
                text: &node.placeholder,
                logical_line: 0,
                is_cursor_line: false,
                cursor_col_in_visual: None,
            });
        }
        cursor_visual_row = Some(0);
    } else {
        // Split into logical lines, preserving trailing empty line
        let mut logical_lines: Vec<&str> = node.value.lines().collect();
        if node.value.ends_with('\n') {
            logical_lines.push("");
        }
        if logical_lines.is_empty() {
            logical_lines.push("");
        }

        for (logical_idx, logical_line) in logical_lines.iter().enumerate() {
            let is_cursor_logical_line = logical_idx == node.cursor_line;

            if content_width == 0 {
                continue;
            }

            // Soft-wrap this logical line
            let wrapped = text::soft_wrap_line(logical_line, content_width);

            for visual_line in wrapped {
                let is_cursor_on_this_visual = is_cursor_logical_line && {
                    // cursor_col is in graphemes within the logical line
                    // Check if cursor falls within this visual line's grapheme range
                    node.cursor_col >= visual_line.grapheme_start
                        && node.cursor_col < visual_line.grapheme_end
                };

                let cursor_col_in_visual = if is_cursor_on_this_visual {
                    // Calculate display column for cursor within this visual line
                    let col_in_visual = node.cursor_col - visual_line.grapheme_start;
                    // Convert grapheme offset to display column
                    let display_col: usize = text::graphemes(visual_line.text)
                        .take(col_in_visual)
                        .map(text::grapheme_width)
                        .sum();
                    cursor_visual_row = Some(visual_lines.len());
                    Some(display_col)
                } else if is_cursor_logical_line && node.cursor_col >= visual_line.grapheme_end {
                    // Cursor might be at end of last visual line of this logical line
                    None
                } else {
                    None
                };

                visual_lines.push(VisualLineInfo {
                    text: visual_line.text,
                    logical_line: logical_idx,
                    is_cursor_line: is_cursor_on_this_visual,
                    cursor_col_in_visual,
                });
            }

            // Handle cursor at end of logical line (past last grapheme)
            if is_cursor_logical_line && cursor_visual_row.is_none() {
                // Cursor is at end of line - mark the last visual line
                if let Some(last) = visual_lines.last_mut() {
                    if last.logical_line == logical_idx {
                        last.is_cursor_line = true;
                        last.cursor_col_in_visual = Some(text::display_width(last.text));
                        cursor_visual_row = Some(visual_lines.len() - 1);
                    }
                }
            }
        }
    }

    // Determine visible rows
    let visible_rows = node.rows.min(area.height) as usize;

    // Calculate scroll offset to keep cursor visible
    let scroll_offset = if let Some(cursor_row) = cursor_visual_row {
        if cursor_row >= visible_rows {
            cursor_row - visible_rows + 1
        } else {
            0
        }
    } else {
        0
    };

    // Border color
    let border_fg = if is_focused {
        theme.border_focused
    } else {
        theme.border
    };

    // Draw content lines
    for row in 0..visible_rows {
        let visual_idx = scroll_offset + row;
        let y = area.y + row as u16;

        // Left border
        buffer.set(area.x, y, '│', border_fg, theme.background);

        // Content
        let (display, is_cursor_line, cursor_col) = if let Some(vl) = visual_lines.get(visual_idx) {
            // Pad to content width by DISPLAY WIDTH, not character count
            // This is critical for wide characters (emoji, CJK)
            let current_display_width = text::display_width(vl.text);
            let padding_needed = content_width.saturating_sub(current_display_width);
            let padded = format!("{}{:padding$}", vl.text, "", padding = padding_needed);
            (padded, vl.is_cursor_line, vl.cursor_col_in_visual)
        } else {
            // Empty line
            (format!("{:width$}", "", width = content_width), false, None)
        };

        let (fg, bg) = if is_focused && is_cursor_line {
            if node.value.is_empty() {
                (theme.placeholder, theme.selection_bg)
            } else {
                (theme.selection_fg, theme.selection_bg)
            }
        } else if node.value.is_empty() {
            (theme.placeholder, theme.input_bg)
        } else {
            (theme.input_fg, theme.input_bg)
        };

        buffer.write_str(area.x + 1, y, &display, fg, bg);

        // Draw cursor if on this line and focused
        if is_focused && is_cursor_line {
            if let Some(col) = cursor_col {
                let cursor_x = (area.x + 1 + col as u16).min(area.x + area.width - 2);
                let cursor_char = display.chars().nth(col).unwrap_or(' ');
                if cursor_char == ' ' {
                    // [end] or [on_space]: show block cursor
                    buffer.set_cell(cursor_x, y, crate::buffer::Cell::new('█', theme.cursor, theme.cursor_text));
                } else {
                    // [on_char]: dark char on bright cursor block (colors swapped)
                    buffer.set_cell(cursor_x, y, crate::buffer::Cell::new(cursor_char, theme.cursor_text, theme.cursor));
                }
            }
        }

        // Right border
        buffer.set(area.x + area.width - 1, y, '│', border_fg, theme.background);
    }
}

fn render_checkbox(buffer: &mut Buffer, node: &CheckboxNode, area: Rect, ctx: &mut RenderContext) {
    // Always increment focusable counter even if we can't render
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }
    let theme = current_theme();

    // Format: [x] label or [ ] label
    let checkbox = if node.checked { "[x]" } else { "[ ]" };
    let display = format!("{} {}", checkbox, node.label);

    let (fg, bg) = if is_focused {
        (theme.selection_fg, theme.selection_bg)
    } else {
        (theme.foreground, theme.background)
    };

    buffer.write_str(area.x, area.y, &display, fg, bg);
}

fn render_radio_group(
    buffer: &mut Buffer,
    node: &RadioGroupNode,
    area: Rect,
    ctx: &mut RenderContext,
) {
    // Always increment focusable counter even if we can't render
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }
    let theme = current_theme();

    // Render optional label first
    let mut y = area.y;
    if let Some(label) = &node.label {
        let (fg, bg) = if is_focused {
            (theme.selection_fg, theme.selection_bg)
        } else {
            (theme.foreground, theme.background)
        };
        buffer.write_str(area.x, y, label, fg, bg);
        y += 1;
    }

    // Render each option
    for (i, option) in node.options.iter().enumerate() {
        if y >= area.y + area.height {
            break;
        }

        let is_selected = i == node.selected;
        let radio = if is_selected { "(●)" } else { "( )" };
        let display = format!("{} {}", radio, option);

        let (fg, bg) = if is_focused && is_selected {
            (theme.selection_fg, theme.selection_bg)
        } else if is_selected {
            (theme.primary, theme.background)
        } else {
            (theme.foreground, theme.background)
        };

        buffer.write_str(area.x, y, &display, fg, bg);
        y += 1;
    }
}

fn render_progress_bar(buffer: &mut Buffer, node: &ProgressBarNode, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Calculate the available width for the bar
    let label_width = node.label.as_ref().map(|l| l.len() + 1).unwrap_or(0) as u16; // +1 for space
    let percentage_width = if node.show_percentage { 5 } else { 0 }; // " 100%" = 5 chars

    // Bar width: fixed if specified, otherwise expand to fill
    let bar_width = node
        .width
        .unwrap_or_else(|| area.width.saturating_sub(label_width + percentage_width))
        .min(area.width.saturating_sub(label_width + percentage_width));

    if bar_width == 0 {
        return;
    }

    let mut x = area.x;

    // Draw label if present
    if let Some(ref label) = node.label {
        buffer.write_str(x, area.y, label, theme.foreground, theme.background);
        x += label.len() as u16 + 1; // +1 for space
    }

    // Calculate filled portion
    let filled_count = ((node.value * bar_width as f32).round() as u16).min(bar_width);
    let empty_count = bar_width.saturating_sub(filled_count);

    // Draw filled portion
    let filled: String = std::iter::repeat_n(node.filled_char, filled_count as usize)
        .collect();
    buffer.write_str(x, area.y, &filled, theme.primary, theme.background);
    x += filled_count;

    // Draw empty portion
    let empty: String = std::iter::repeat_n(node.empty_char, empty_count as usize)
        .collect();
    buffer.write_str(x, area.y, &empty, theme.border, theme.background);
    x += empty_count;

    // Draw percentage if enabled
    if node.show_percentage {
        let percentage = format!(" {:3.0}%", node.value * 100.0);
        buffer.write_str(x, area.y, &percentage, theme.foreground, theme.background);
    }
}

fn render_status_bar(buffer: &mut Buffer, node: &StatusBarNode, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Use custom colors or fall back to theme
    let fg = node.fg_color.unwrap_or(theme.foreground);
    let bg = node.bg_color.unwrap_or(theme.secondary);

    // Fill the entire status bar area with background
    let spaces: String = " ".repeat(area.width as usize);
    buffer.write_str(area.x, area.y, &spaces, fg, bg);

    // Calculate widths
    let left_width = node.left.len();
    let center_width = node.center.as_ref().map(|c| c.len()).unwrap_or(0);
    let right_width = node.right.as_ref().map(|r| r.len()).unwrap_or(0);
    let total_width = area.width as usize;

    // Draw left section (always at x=0)
    if left_width > 0 {
        let left_display: String = if left_width > total_width {
            node.left.chars().take(total_width).collect()
        } else {
            node.left.clone()
        };
        buffer.write_str(area.x, area.y, &left_display, fg, bg);
    }

    // Draw center section (centered in available space)
    if center_width > 0 {
        if let Some(ref center) = node.center {
            let center_x = (total_width.saturating_sub(center_width)) / 2;
            // Only draw if it doesn't overlap with left
            if center_x >= left_width {
                let display: String = center
                    .chars()
                    .take(total_width.saturating_sub(center_x))
                    .collect();
                buffer.write_str(area.x + center_x as u16, area.y, &display, fg, bg);
            }
        }
    }

    // Draw right section (right-aligned)
    if right_width > 0 {
        if let Some(ref right) = node.right {
            let right_x = total_width.saturating_sub(right_width);
            // Only draw if there's space
            if right_x > 0 {
                buffer.write_str(area.x + right_x as u16, area.y, right, fg, bg);
            }
        }
    }
}

fn render_tabs(buffer: &mut Buffer, node: &TabsNode, area: Rect, ctx: &mut RenderContext) {
    // Tabs widget is focusable for tab switching
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let _theme = current_theme();
    let tab_bar_height = 1u16;

    // Determine areas based on position
    let (tab_bar_area, content_area) = match node.position {
        TabPosition::Top => (
            Rect::new(area.x, area.y, area.width, tab_bar_height),
            Rect::new(
                area.x,
                area.y + tab_bar_height,
                area.width,
                area.height.saturating_sub(tab_bar_height),
            ),
        ),
        TabPosition::Bottom => (
            Rect::new(
                area.x,
                area.y + area.height.saturating_sub(tab_bar_height),
                area.width,
                tab_bar_height,
            ),
            Rect::new(
                area.x,
                area.y,
                area.width,
                area.height.saturating_sub(tab_bar_height),
            ),
        ),
    };

    // Render tab bar
    render_tab_bar(buffer, &node.tabs, node.active, tab_bar_area, is_focused);

    // Render active tab content
    if node.active < node.children.len() {
        render_view(buffer, &node.children[node.active], content_area, ctx);
    }
}

fn render_tab_bar(
    buffer: &mut Buffer,
    tabs: &[String],
    active: usize,
    area: Rect,
    is_focused: bool,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();
    let mut x = area.x;

    for (i, tab) in tabs.iter().enumerate() {
        let is_active = i == active;

        // Format: " Tab " with separator
        let label = format!(" {} ", tab);
        let label_width = label.len() as u16;

        if x + label_width > area.x + area.width {
            break; // No more space for tabs
        }

        let (fg, bg) = if is_active {
            if is_focused {
                (theme.selection_fg, theme.selection_bg)
            } else {
                (theme.primary, theme.background)
            }
        } else {
            (theme.muted, theme.background)
        };

        buffer.write_str(x, area.y, &label, fg, bg);
        x += label_width;

        // Draw separator
        if i < tabs.len() - 1 && x < area.x + area.width {
            buffer.set(x, area.y, '│', theme.muted, theme.background);
            x += 1;
        }
    }

    // Fill remaining space with underline if at top
    for fill_x in x..area.x + area.width {
        buffer.set(fill_x, area.y, ' ', theme.foreground, theme.background);
    }
}

fn render_tree(buffer: &mut Buffer, node: &TreeNode, area: Rect, ctx: &mut RenderContext) {
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Flatten tree to get visible items with their paths and depths
    let visible = flatten_tree_for_render(&node.items, &[]);

    // Render visible items within the area
    for (row, (path, depth, item)) in visible.iter().enumerate() {
        if row as u16 >= area.height {
            break;
        }

        let y = area.y + row as u16;
        let is_selected = *path == node.selected;

        // Build the line: indent + expand marker + icon + label
        let indent = "  ".repeat(*depth);
        let expand_marker = if item.children.is_empty() {
            "  " // Leaf node - no marker
        } else if item.expanded {
            "▼ "
        } else {
            "▶ "
        };
        let icon = item.icon.as_deref().unwrap_or("");
        let icon_space = if icon.is_empty() { "" } else { " " };

        let line = format!(
            "{}{}{}{}{}",
            indent, expand_marker, icon, icon_space, item.label
        );

        // Truncate or pad to fit width
        let display: String = if line.chars().count() > area.width as usize {
            line.chars().take(area.width as usize).collect()
        } else {
            format!("{:width$}", line, width = area.width as usize)
        };

        let (fg, bg) = if is_selected && is_focused {
            (theme.selection_fg, theme.selection_bg)
        } else if is_selected {
            (theme.primary, theme.background)
        } else {
            (theme.foreground, theme.background)
        };

        buffer.write_str(area.x, y, &display, fg, bg);
    }

    // Fill remaining rows if tree is shorter than area
    for row in visible.len()..area.height as usize {
        let y = area.y + row as u16;
        let blank = " ".repeat(area.width as usize);
        buffer.write_str(area.x, y, &blank, theme.foreground, theme.background);
    }
}

/// Helper to flatten tree items for rendering.
fn flatten_tree_for_render<'a>(
    items: &'a [TreeItem],
    base_path: &[usize],
) -> Vec<(TreePath, usize, &'a TreeItem)> {
    let mut result = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let mut path = base_path.to_vec();
        path.push(i);
        let depth = path.len() - 1;
        result.push((path.clone(), depth, item));
        if item.expanded && !item.children.is_empty() {
            result.extend(flatten_tree_for_render(&item.children, &path));
        }
    }
    result
}

fn render_split(buffer: &mut Buffer, node: &SplitNode, area: Rect, ctx: &mut RenderContext) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();
    let divider_size = if node.show_divider { 1 } else { 0 };

    // Calculate pane sizes based on orientation
    let (first_area, second_area) = match node.orientation {
        Orientation::Horizontal => {
            // Side by side: [first | second]
            let total_width = area.width.saturating_sub(divider_size);
            let first_width =
                calculate_split_size(total_width, node.ratio, node.min_first, node.min_second);
            let second_width = total_width.saturating_sub(first_width);

            let first = Rect::new(area.x, area.y, first_width, area.height);
            let second = Rect::new(
                area.x + first_width + divider_size,
                area.y,
                second_width,
                area.height,
            );

            // Draw vertical divider
            if node.show_divider && first_width < area.width {
                let divider_x = area.x + first_width;
                for y in area.y..area.y + area.height {
                    buffer.set(divider_x, y, '│', theme.muted, theme.background);
                }
            }

            (first, second)
        }
        Orientation::Vertical => {
            // Stacked: [first] / [second]
            let total_height = area.height.saturating_sub(divider_size);
            let first_height =
                calculate_split_size(total_height, node.ratio, node.min_first, node.min_second);
            let second_height = total_height.saturating_sub(first_height);

            let first = Rect::new(area.x, area.y, area.width, first_height);
            let second = Rect::new(
                area.x,
                area.y + first_height + divider_size,
                area.width,
                second_height,
            );

            // Draw horizontal divider
            if node.show_divider && first_height < area.height {
                let divider_y = area.y + first_height;
                for x in area.x..area.x + area.width {
                    buffer.set(x, divider_y, '─', theme.muted, theme.background);
                }
            }

            (first, second)
        }
    };

    // Render first pane
    render_view(buffer, &node.first, first_area, ctx);

    // Render second pane
    render_view(buffer, &node.second, second_area, ctx);
}

/// Calculate the size of the first pane in a split, respecting minimums.
fn calculate_split_size(
    total: u16,
    ratio: f32,
    min_first: Option<u16>,
    min_second: Option<u16>,
) -> u16 {
    let desired_first = (total as f32 * ratio) as u16;

    // Apply minimum constraints
    let min_first = min_first.unwrap_or(0);
    let min_second = min_second.unwrap_or(0);

    // Ensure first pane is at least min_first
    let first = desired_first.max(min_first);

    // Ensure second pane has at least min_second
    let max_first = total.saturating_sub(min_second);
    first.min(max_first)
}

fn render_modal(buffer: &mut Buffer, node: &ModalNode, _area: Rect, ctx: &mut RenderContext) {
    // Don't render if not visible
    if !node.visible {
        return;
    }

    let theme = current_theme();

    // Use root area (full screen) for modal positioning, not the passed-in area
    let area = ctx.root_area;

    // Calculate modal dimensions based on percentage
    let modal_width = (area.width as u32 * node.width_percent as u32 / 100) as u16;
    let modal_height = (area.height as u32 * node.height_percent as u32 / 100) as u16;

    // Minimum size
    let modal_width = modal_width.max(20);
    let modal_height = modal_height.max(5);

    // Center the modal
    let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Draw semi-transparent backdrop (dim the background)
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            // Skip the modal area
            if x >= modal_x
                && x < modal_x + modal_width
                && y >= modal_y
                && y < modal_y + modal_height
            {
                continue;
            }
            // Dim the existing cell
            if let Some(cell) = buffer.get(x, y) {
                let mut dimmed = *cell;
                dimmed.dim = true;
                buffer.set_cell(x, y, dimmed);
            }
        }
    }

    // Clear the modal area with background
    for y in modal_y..modal_y + modal_height {
        for x in modal_x..modal_x + modal_width {
            buffer.set(x, y, ' ', theme.foreground, theme.background);
        }
    }

    // Draw modal border
    draw_border_focused(buffer, modal_area);

    // Draw title in the top border if provided
    if !node.title.is_empty() {
        let title = format!(" {} ", node.title);
        let title_x = modal_x + 2;
        if title.len() < (modal_width - 4) as usize {
            buffer.write_str(title_x, modal_y, &title, theme.primary, theme.background);
        }
    }

    // Render child content inside the modal
    if let Some(child) = &node.child {
        let content_area = Rect::new(
            modal_x + 1,
            modal_y + 1,
            modal_width.saturating_sub(2),
            modal_height.saturating_sub(2),
        );
        // Mark that we're inside the modal so focusables get counted
        ctx.inside_modal = true;
        render_view(buffer, child, content_area, ctx);
        ctx.inside_modal = false;
    }
}

fn render_table(buffer: &mut Buffer, node: &TableNode, area: Rect, ctx: &mut RenderContext) {
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();
    let col_count = node.columns.len();

    if col_count == 0 {
        return;
    }

    // Calculate column widths
    let col_widths = calculate_column_widths(&node.columns, &node.rows, area.width);

    // Row 0: Header
    let header_y = area.y;
    let mut x = area.x;

    for (col_idx, col) in node.columns.iter().enumerate() {
        let width = col_widths[col_idx] as usize;
        if width == 0 {
            continue;
        }

        // Build header text with sort indicator
        let sort_indicator = match node.sort {
            Some((sort_col, asc)) if sort_col == col_idx => {
                if asc {
                    " ▲"
                } else {
                    " ▼"
                }
            }
            _ => "",
        };

        let header_text = format!("{}{}", col.header, sort_indicator);
        let display = align_text(&header_text, width, col.align);

        // Header styling
        buffer.write_str_styled(
            x,
            header_y,
            &display,
            theme.primary,
            theme.background,
            true, // bold
            false,
            false,
            false,
        );

        x += width as u16;

        // Draw column separator
        if col_idx < col_count - 1 && x < area.x + area.width {
            buffer.set(x, header_y, '│', theme.muted, theme.background);
            x += 1;
        }
    }

    // Row 1: Header separator line
    if area.height > 1 {
        let sep_y = area.y + 1;
        x = area.x;

        for (col_idx, _col) in node.columns.iter().enumerate() {
            let width = col_widths[col_idx] as usize;

            for _ in 0..width {
                if x < area.x + area.width {
                    buffer.set(x, sep_y, '─', theme.muted, theme.background);
                    x += 1;
                }
            }

            // Draw intersection
            if col_idx < col_count - 1 && x < area.x + area.width {
                buffer.set(x, sep_y, '┼', theme.muted, theme.background);
                x += 1;
            }
        }
    }

    // Data rows (starting at row 2)
    let data_start_y = area.y + 2;
    let visible_rows = area.height.saturating_sub(2) as usize;

    // Calculate scroll offset to keep selected row visible
    let scroll_offset = if node.selected >= visible_rows && visible_rows > 0 {
        node.selected - visible_rows + 1
    } else {
        0
    };

    for row_idx in 0..visible_rows {
        let data_idx = scroll_offset + row_idx;
        if data_idx >= node.rows.len() {
            break;
        }

        let y = data_start_y + row_idx as u16;
        if y >= area.y + area.height {
            break;
        }

        let row_data = &node.rows[data_idx];
        let is_selected = data_idx == node.selected;

        let (row_fg, row_bg) = if is_selected && is_focused {
            (theme.selection_fg, theme.selection_bg)
        } else if is_selected {
            (theme.primary, theme.background)
        } else {
            (theme.foreground, theme.background)
        };

        x = area.x;

        for (col_idx, col) in node.columns.iter().enumerate() {
            let width = col_widths[col_idx] as usize;
            if width == 0 {
                continue;
            }

            let cell_text = row_data.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            let display = align_text(cell_text, width, col.align);

            buffer.write_str(x, y, &display, row_fg, row_bg);
            x += width as u16;

            // Draw column separator
            if col_idx < col_count - 1 && x < area.x + area.width {
                buffer.set(x, y, '│', theme.muted, row_bg);
                x += 1;
            }
        }

        // Fill remaining width if row is shorter
        while x < area.x + area.width {
            buffer.set(x, y, ' ', row_fg, row_bg);
            x += 1;
        }
    }

    // Fill remaining rows
    for row_idx in node.rows.len().saturating_sub(scroll_offset)..visible_rows {
        let y = data_start_y + row_idx as u16;
        if y >= area.y + area.height {
            break;
        }
        for fill_x in area.x..area.x + area.width {
            buffer.set(fill_x, y, ' ', theme.foreground, theme.background);
        }
    }
}

/// Calculate column widths based on column specifications and available space.
fn calculate_column_widths(
    columns: &[crate::view::TableColumn],
    rows: &[Vec<String>],
    total_width: u16,
) -> Vec<u16> {
    let col_count = columns.len();
    if col_count == 0 {
        return vec![];
    }

    // Account for separators between columns
    let separator_width = (col_count - 1) as u16;
    let available_width = total_width.saturating_sub(separator_width);

    let mut widths: Vec<u16> = vec![0; col_count];
    let mut total_fixed: u16 = 0;
    let mut total_flex: u16 = 0;

    // First pass: handle Fixed and Auto widths, count Flex
    for (i, col) in columns.iter().enumerate() {
        match col.width {
            ColumnWidth::Fixed(w) => {
                widths[i] = w;
                total_fixed += w;
            }
            ColumnWidth::Auto => {
                // Calculate auto width from content
                let header_len = col.header.len() as u16 + 2; // +2 for potential sort indicator
                let max_content: u16 = rows
                    .iter()
                    .filter_map(|row| row.get(i))
                    .map(|s| s.len() as u16)
                    .max()
                    .unwrap_or(0);
                let auto_width = header_len.max(max_content).min(30); // Cap auto width at 30
                widths[i] = auto_width;
                total_fixed += auto_width;
            }
            ColumnWidth::Flex(factor) => {
                total_flex += factor;
            }
        }
    }

    // Second pass: distribute remaining space to Flex columns
    let flex_space = available_width.saturating_sub(total_fixed);

    if total_flex > 0 {
        for (i, col) in columns.iter().enumerate() {
            if let ColumnWidth::Flex(factor) = col.width {
                widths[i] = (flex_space * factor) / total_flex;
            }
        }
    }

    // Ensure each column has at least width 1 if we have space
    for w in &mut widths {
        if *w == 0 && total_width > 0 {
            *w = 1;
        }
    }

    widths
}

/// Align text within a given width according to the specified alignment.
fn align_text(text: &str, width: usize, align: TextAlign) -> String {
    let text_len = text.chars().count();

    if text_len >= width {
        // Truncate if too long
        text.chars().take(width).collect()
    } else {
        let padding = width - text_len;
        match align {
            TextAlign::Left => format!("{}{}", text, " ".repeat(padding)),
            TextAlign::Right => format!("{}{}", " ".repeat(padding), text),
            TextAlign::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
            }
        }
    }
}

// =============================================================================
// Command Palette
// =============================================================================

fn render_command_palette(
    buffer: &mut Buffer,
    node: &CommandPaletteNode,
    _area: Rect,
    ctx: &mut RenderContext,
) {
    if !node.visible {
        return;
    }

    // Always increment focusable counter when visible
    let is_focused = ctx.is_next_focused();

    let theme = current_theme();

    // Use root area for overlay positioning
    let area = ctx.root_area;

    // Calculate palette dimensions
    let palette_width = (area.width as u32 * node.width_percent as u32 / 100) as u16;
    let palette_height = (area.height as u32 * node.height_percent as u32 / 100) as u16;

    // Minimum size
    let palette_width = palette_width.max(30).min(area.width);
    let palette_height = palette_height.max(5).min(area.height);

    // Center horizontally, position near top
    let palette_x = area.x + (area.width.saturating_sub(palette_width)) / 2;
    let palette_y = area.y + 2; // A bit below top

    let palette_area = Rect::new(palette_x, palette_y, palette_width, palette_height);

    // Draw semi-transparent backdrop
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            if x >= palette_x
                && x < palette_x + palette_width
                && y >= palette_y
                && y < palette_y + palette_height
            {
                continue;
            }
            if let Some(cell) = buffer.get(x, y) {
                let mut dimmed = *cell;
                dimmed.dim = true;
                buffer.set_cell(x, y, dimmed);
            }
        }
    }

    // Clear palette area
    for y in palette_y..palette_y + palette_height {
        for x in palette_x..palette_x + palette_width {
            buffer.set(x, y, ' ', theme.foreground, theme.background);
        }
    }

    // Draw border
    draw_border_focused(buffer, palette_area);

    // Draw title
    let title = " Command Palette ";
    let title_x = palette_x + 2;
    buffer.write_str(title_x, palette_y, title, theme.primary, theme.background);

    // Draw search input
    let input_y = palette_y + 1;
    let input_width = (palette_width - 4) as usize;
    let query_display = if node.query.is_empty() {
        "Type to search...".to_string()
    } else {
        node.query.clone()
    };
    let query_truncated: String = query_display.chars().take(input_width).collect();
    let query_padded = format!("{:<width$}", query_truncated, width = input_width);

    let (query_fg, query_bg) = if is_focused {
        (theme.selection_fg, theme.selection_bg)
    } else if node.query.is_empty() {
        (theme.placeholder, theme.input_bg)
    } else {
        (theme.input_fg, theme.input_bg)
    };
    buffer.write_str(palette_x + 2, input_y, &query_padded, query_fg, query_bg);

    // Draw cursor
    // Cursor scenarios: end, on_space → '█'; on_char → char (all with cursor/cursor_text)
    if is_focused {
        let cursor_pos = node.query.len().min(input_width);
        let cursor_x = palette_x + 2 + cursor_pos as u16;
        if cursor_x < palette_x + palette_width - 2 {
            let cursor_char = query_padded.chars().nth(cursor_pos).unwrap_or(' ');
            if cursor_char == ' ' {
                // [end] or [on_space]: show block cursor
                buffer.set_cell(cursor_x, input_y, crate::buffer::Cell::new('█', theme.cursor, theme.cursor_text));
            } else {
                // [on_char]: dark char on bright cursor block (colors swapped)
                buffer.set_cell(cursor_x, input_y, crate::buffer::Cell::new(cursor_char, theme.cursor_text, theme.cursor));
            }
        }
    }

    // Draw separator
    let sep_y = palette_y + 2;
    for x in palette_x + 1..palette_x + palette_width - 1 {
        buffer.set(x, sep_y, '─', theme.border, theme.background);
    }

    // Filter commands
    let filtered = filter_palette_commands(&node.commands, &node.query);

    // Draw command list
    let list_y = sep_y + 1;
    let list_height = palette_height.saturating_sub(4) as usize;
    let content_width = (palette_width - 4) as usize;

    for (i, cmd) in filtered.iter().take(list_height).enumerate() {
        let y = list_y + i as u16;
        let is_selected = i == node.selected;

        // Format: label ... shortcut
        let shortcut = cmd.shortcut.as_deref().unwrap_or("");
        let shortcut_len = shortcut.len();
        let label_max_width = content_width.saturating_sub(shortcut_len + 2);
        let label: String = cmd.label.chars().take(label_max_width).collect();
        let gap = content_width.saturating_sub(label.len() + shortcut_len);

        let line = format!("{}{}{}", label, " ".repeat(gap), shortcut);

        let (fg, bg) = if is_selected {
            (theme.selection_fg, theme.selection_bg)
        } else {
            (theme.foreground, theme.background)
        };

        buffer.write_str(palette_x + 2, y, &line, fg, bg);
    }

    // Draw hint at bottom
    if palette_height > 5 {
        let hint_y = palette_y + palette_height - 1;
        let hint = "↑↓ navigate • Enter select • Esc cancel";
        let hint_x = palette_x + 2;
        let hint_display: String = hint.chars().take(content_width).collect();
        buffer.write_str(hint_x, hint_y, &hint_display, theme.muted, theme.background);
    }
}

/// Filter palette commands by query.
fn filter_palette_commands<'a>(
    commands: &'a [PaletteCommand],
    query: &str,
) -> Vec<&'a PaletteCommand> {
    if query.is_empty() {
        return commands.iter().collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches: Vec<(&PaletteCommand, i32)> = commands
        .iter()
        .filter_map(|cmd| {
            let score = palette_fuzzy_score(&cmd.label.to_lowercase(), &query_lower);
            if score > 0 {
                Some((cmd, score))
            } else {
                None
            }
        })
        .collect();

    matches.sort_by(|a, b| b.1.cmp(&a.1));
    matches.into_iter().map(|(cmd, _)| cmd).collect()
}

/// Simple fuzzy score for palette filtering.
fn palette_fuzzy_score(text: &str, query: &str) -> i32 {
    if query.is_empty() {
        return 1;
    }

    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    let mut text_idx = 0;
    let mut query_idx = 0;
    let mut score = 0;
    let mut consecutive = 0;

    while text_idx < text_chars.len() && query_idx < query_chars.len() {
        if text_chars[text_idx] == query_chars[query_idx] {
            consecutive += 1;
            score += consecutive * 2;
            if text_idx == 0 || !text_chars[text_idx - 1].is_alphanumeric() {
                score += 5;
            }
            query_idx += 1;
        } else {
            consecutive = 0;
        }
        text_idx += 1;
    }

    if query_idx == query_chars.len() {
        score
    } else {
        0
    }
}

// =============================================================================
// Menu Bar
// =============================================================================

fn render_menu_bar(buffer: &mut Buffer, node: &MenuBarNode, area: Rect, ctx: &mut RenderContext) {
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Fill background
    for x in area.x..area.x + area.width {
        buffer.set(x, area.y, ' ', theme.foreground, theme.secondary);
    }

    // Draw menu labels
    let mut x = area.x;
    for (i, menu) in node.menus.iter().enumerate() {
        let is_active = node.active_menu == Some(i);
        let label = format!(" {} ", menu.label);
        let label_width = label.len() as u16;

        if x + label_width > area.x + area.width {
            break;
        }

        let (fg, bg) = if is_active {
            (theme.selection_fg, theme.selection_bg)
        } else if is_focused && i == node.highlighted_menu {
            (theme.primary, theme.secondary)
        } else {
            (theme.foreground, theme.secondary)
        };

        buffer.write_str(x, area.y, &label, fg, bg);
        x += label_width;
    }

    // Queue dropdown for active menu (rendered as overlay after main content)
    if let Some(active_idx) = node.active_menu {
        if let Some(menu) = node.menus.get(active_idx) {
            // Calculate dropdown position
            let dropdown_x = node
                .menus
                .iter()
                .take(active_idx)
                .map(|m| m.label.len() + 2)
                .sum::<usize>() as u16
                + area.x;

            ctx.queue_dropdown(menu.clone(), dropdown_x, area.y + 1, node.selected_item);
        }
    }
}

fn render_menu_dropdown_impl(
    buffer: &mut Buffer,
    menu: &crate::view::Menu,
    x: u16,
    y: u16,
    selected: usize,
    ctx: &RenderContext,
) {
    let theme = current_theme();

    // Calculate dropdown dimensions
    let mut max_width: usize = 20;
    for item in &menu.items {
        match item {
            MenuItemNode::Command {
                label, shortcut, ..
            } => {
                let shortcut_len = shortcut.as_ref().map(|s| s.len() + 2).unwrap_or(0);
                max_width = max_width.max(label.len() + shortcut_len + 4);
            }
            MenuItemNode::Separator => {}
        }
    }

    let dropdown_width = max_width as u16;
    let dropdown_height = menu.items.len() as u16 + 2; // +2 for border

    // Ensure dropdown fits on screen
    let max_x = ctx.root_area.x + ctx.root_area.width;
    let max_y = ctx.root_area.y + ctx.root_area.height;
    let dropdown_x = x.min(max_x.saturating_sub(dropdown_width));
    let dropdown_y = y.min(max_y.saturating_sub(dropdown_height));

    let dropdown_area = Rect::new(dropdown_x, dropdown_y, dropdown_width, dropdown_height);

    // Clear and draw border
    for dy in 0..dropdown_height {
        for dx in 0..dropdown_width {
            buffer.set(
                dropdown_x + dx,
                dropdown_y + dy,
                ' ',
                theme.foreground,
                theme.background,
            );
        }
    }
    draw_border(buffer, dropdown_area);

    // Draw items
    let mut cmd_idx = 0usize;
    for (row, item) in menu.items.iter().enumerate() {
        let item_y = dropdown_y + 1 + row as u16;
        if item_y >= dropdown_y + dropdown_height - 1 {
            break;
        }

        match item {
            MenuItemNode::Command {
                label, shortcut, ..
            } => {
                let is_selected = cmd_idx == selected;
                cmd_idx += 1;

                // Format: " label    shortcut "
                let shortcut_str = shortcut.as_deref().unwrap_or("");
                let content_width = (dropdown_width - 2) as usize;
                let gap = content_width.saturating_sub(label.len() + shortcut_str.len() + 1);
                let line = format!("{}{}{}", label, " ".repeat(gap), shortcut_str);
                let line: String = line.chars().take(content_width).collect();

                let (fg, bg) = if is_selected {
                    (theme.selection_fg, theme.selection_bg)
                } else {
                    (theme.foreground, theme.background)
                };

                buffer.write_str(dropdown_x + 1, item_y, &line, fg, bg);
            }
            MenuItemNode::Separator => {
                // Draw a horizontal line
                for dx in 1..dropdown_width - 1 {
                    buffer.set(dropdown_x + dx, item_y, '─', theme.border, theme.background);
                }
            }
        }
    }
}

// =============================================================================
// Toast Container
// =============================================================================

fn render_toast_container(buffer: &mut Buffer, node: &ToastContainerNode, ctx: &RenderContext) {
    if node.toasts.is_empty() {
        return;
    }

    let theme = current_theme();
    let area = ctx.root_area;

    // Calculate toast dimensions
    let toast_width = node.width.min(area.width.saturating_sub(2));
    let toast_height = 3u16; // Border + content + border

    // Visible toasts (limited by max_visible)
    let visible_toasts: Vec<_> = node.toasts.iter().take(node.max_visible).collect();

    // Calculate starting position based on corner
    let (start_x, start_y, direction) = match node.position {
        ToastPosition::TopRight => {
            let x = area.x + area.width - toast_width - 1;
            let y = area.y + 1;
            (x, y, 1i16) // direction: positive = going down
        }
        ToastPosition::TopLeft => {
            let x = area.x + 1;
            let y = area.y + 1;
            (x, y, 1)
        }
        ToastPosition::BottomRight => {
            let x = area.x + area.width - toast_width - 1;
            let y = area.y + area.height - toast_height - 1;
            (x, y, -1i16) // direction: negative = going up
        }
        ToastPosition::BottomLeft => {
            let x = area.x + 1;
            let y = area.y + area.height - toast_height - 1;
            (x, y, -1)
        }
    };

    // Render each toast
    for (i, toast) in visible_toasts.iter().enumerate() {
        let offset = (i as i16) * (toast_height as i16 + 1) * direction;
        let toast_y = (start_y as i16 + offset) as u16;

        // Skip if off screen
        if toast_y + toast_height > area.y + area.height || toast_y < area.y {
            continue;
        }

        render_toast(
            buffer,
            toast,
            start_x,
            toast_y,
            toast_width,
            toast_height,
            &theme,
        );
    }
}

fn render_toast(
    buffer: &mut Buffer,
    toast: &crate::view::ToastItem,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    theme: &crate::theme::Theme,
) {
    // Choose colors based on level
    let (fg, bg, border_fg) = match toast.level {
        ToastLevelView::Info => (theme.foreground, theme.background, theme.border),
        ToastLevelView::Success => (
            theme.foreground,
            theme.background,
            crossterm::style::Color::Green,
        ),
        ToastLevelView::Warning => (
            theme.foreground,
            theme.background,
            crossterm::style::Color::Yellow,
        ),
        ToastLevelView::Error => (
            theme.foreground,
            theme.background,
            crossterm::style::Color::Red,
        ),
    };

    let toast_area = Rect::new(x, y, width, height);

    // Clear toast area
    for dy in 0..height {
        for dx in 0..width {
            buffer.set(x + dx, y + dy, ' ', fg, bg);
        }
    }

    // Draw border with level-appropriate color
    draw_toast_border(buffer, toast_area, border_fg, bg);

    // Draw icon based on level
    let icon = match toast.level {
        ToastLevelView::Info => "ℹ",
        ToastLevelView::Success => "✓",
        ToastLevelView::Warning => "⚠",
        ToastLevelView::Error => "✗",
    };

    buffer.write_str(x + 1, y + 1, icon, border_fg, bg);

    // Draw message
    let content_width = (width - 4) as usize; // -4 for borders and icon
    let message: String = toast.message.chars().take(content_width).collect();
    buffer.write_str(x + 3, y + 1, &message, fg, bg);

    // Draw progress indicator (fade effect)
    if toast.progress < 1.0 {
        let progress_width = ((width - 2) as f32 * toast.progress) as u16;
        for dx in 0..progress_width {
            buffer.set(x + 1 + dx, y + height - 1, '─', border_fg, bg);
        }
    }
}

fn draw_toast_border(
    buffer: &mut Buffer,
    area: Rect,
    fg: crossterm::style::Color,
    bg: crossterm::style::Color,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    // Corners
    buffer.set(area.x, area.y, '╭', fg, bg);
    buffer.set(area.x + area.width - 1, area.y, '╮', fg, bg);
    buffer.set(area.x, area.y + area.height - 1, '╰', fg, bg);
    buffer.set(
        area.x + area.width - 1,
        area.y + area.height - 1,
        '╯',
        fg,
        bg,
    );

    // Top and bottom edges
    for x in (area.x + 1)..(area.x + area.width - 1) {
        buffer.set(x, area.y, '─', fg, bg);
        buffer.set(x, area.y + area.height - 1, '─', fg, bg);
    }

    // Left and right edges
    for y in (area.y + 1)..(area.y + area.height - 1) {
        buffer.set(area.x, y, '│', fg, bg);
        buffer.set(area.x + area.width - 1, y, '│', fg, bg);
    }
}

// =============================================================================
// Form
// =============================================================================

fn render_form(buffer: &mut Buffer, node: &FormNode, area: Rect, ctx: &mut RenderContext) {
    if node.children.is_empty() || area.height == 0 {
        return;
    }

    // Render children vertically with spacing
    let mut y = area.y;

    for (i, child) in node.children.iter().enumerate() {
        let child_height = child.intrinsic_height().unwrap_or(3);

        if y + child_height > area.y + area.height {
            break;
        }

        let child_area = Rect::new(area.x, y, area.width, child_height);
        render_view(buffer, child, child_area, ctx);

        y += child_height;

        // Add spacing between children
        if i < node.children.len() - 1 {
            y += node.spacing;
        }
    }
}

fn render_form_field(
    buffer: &mut Buffer,
    node: &FormFieldNode,
    area: Rect,
    ctx: &mut RenderContext,
) {
    // Use is_next_focused_for_cursor: form fields must show cursor even before Tab is pressed
    let is_focused = ctx.is_next_focused_for_cursor();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Row 0: Label
    let label_fg = if node.error.is_some() {
        crossterm::style::Color::Red
    } else {
        theme.foreground
    };
    buffer.write_str(area.x, area.y, &node.label, label_fg, theme.background);

    // Row 1: Input field
    if area.height > 1 {
        let input_y = area.y + 1;
        let input_width = area.width as usize;

        // Calculate visible portion based on cursor
        let cursor_pos = node.cursor_pos.min(node.value.len());
        let display_value = if node.password {
            "*".repeat(node.value.len())
        } else {
            node.value.clone()
        };

        // Cursor scenarios: end, on_space → '█'; on_char → char (all with cursor/cursor_text)
        // placeholder: when empty, no placeholder inversion - just cursor
        let (visible_text, cursor_offset) = if display_value.is_empty() {
            if is_focused {
                // [placeholder] When focused and empty, don't show placeholder - just cursor
                (String::new(), 0)
            } else {
                // When unfocused and empty, show placeholder
                (node.placeholder.clone(), 0)
            }
        } else if display_value.len() <= input_width {
            (display_value, cursor_pos)
        } else if cursor_pos < input_width {
            (display_value[..input_width].to_string(), cursor_pos)
        } else {
            let start = cursor_pos.saturating_sub(input_width - 1);
            let end = (start + input_width).min(display_value.len());
            (display_value[start..end].to_string(), cursor_pos - start)
        };

        // Build display string
        let padding = input_width.saturating_sub(visible_text.len());
        let content = format!("{}{}", visible_text, " ".repeat(padding));

        let (fg, bg) = if node.error.is_some() {
            (crossterm::style::Color::Red, theme.input_bg)
        } else if is_focused {
            (theme.selection_fg, theme.selection_bg)
        } else if node.value.is_empty() {
            (theme.placeholder, theme.input_bg)
        } else {
            (theme.input_fg, theme.input_bg)
        };

        buffer.write_str(area.x, input_y, &content, fg, bg);

        // Draw cursor when focused
        if is_focused {
            let cursor_x = area.x + cursor_offset as u16;
            if cursor_x < area.x + area.width {
                let cursor_char = content.chars().nth(cursor_offset).unwrap_or(' ');
                if cursor_char == ' ' {
                    // [end] or [on_space]: show block cursor
                    buffer.set_cell(cursor_x, input_y, crate::buffer::Cell::new('█', theme.cursor, theme.cursor_text));
                } else {
                    // [on_char]: dark char on bright cursor block (colors swapped)
                    buffer.set_cell(cursor_x, input_y, crate::buffer::Cell::new(cursor_char, theme.cursor_text, theme.cursor));
                }
            }
        }
    }

    // Row 2: Error message (if present)
    if area.height > 2 {
        if let Some(ref error) = node.error {
            let error_y = area.y + 2;
            let error_display: String = error.chars().take(area.width as usize).collect();
            buffer.write_str(
                area.x,
                error_y,
                &error_display,
                crossterm::style::Color::Red,
                theme.background,
            );
        }
    }
}

// =============================================================================
// Canvas Rendering
// =============================================================================

fn render_canvas(_buffer: &mut Buffer, node: &CanvasNode, area: Rect, ctx: &mut RenderContext) {
    use crate::canvas::{DrawContext, PendingCanvas, PixelBuffer};

    if area.width == 0 || area.height == 0 {
        return;
    }

    // Check if Kitty graphics is supported
    if !crate::canvas::supports_kitty_graphics() {
        // Unsupported terminal - render a placeholder
        let theme = current_theme();
        let placeholder = "[Canvas: requires Kitty/Ghostty/WezTerm]";
        let display: String = placeholder.chars().take(area.width as usize).collect();
        _buffer.write_str(
            area.x,
            area.y,
            &display,
            theme.placeholder,
            theme.background,
        );
        return;
    }

    // Create pixel buffer
    let mut pixels = PixelBuffer::new(node.pixel_width, node.pixel_height);

    // Call user's draw callback
    if let Some(ref on_draw) = node.on_draw {
        let mut draw_ctx = DrawContext::new(&mut pixels);
        on_draw(&mut draw_ctx);
    }

    // Queue the canvas for rendering via Kitty protocol
    ctx.queue_canvas(PendingCanvas {
        cell_x: area.x,
        cell_y: area.y,
        pixels,
        id: node.id,
    });

    // Fill the character buffer area with spaces (canvas will overlay via Kitty)
    // This ensures no character content shows through
    let theme = current_theme();
    for dy in 0..area.height {
        let row_str = " ".repeat(area.width as usize);
        _buffer.write_str(
            area.x,
            area.y + dy,
            &row_str,
            theme.foreground,
            theme.background,
        );
    }
}

// =============================================================================
// Image Rendering
// =============================================================================

fn render_image(_buffer: &mut Buffer, node: &ImageNode, area: Rect, ctx: &mut RenderContext) {
    use crate::image::{ImageSource, PendingImage};

    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Check if Kitty graphics is supported
    if !crate::canvas::supports_kitty_graphics() {
        // Unsupported terminal - render alt text or placeholder
        let placeholder = node
            .alt
            .as_deref()
            .unwrap_or("[Image: requires Kitty/Ghostty/WezTerm]");
        let display: String = placeholder.chars().take(area.width as usize).collect();
        _buffer.write_str(
            area.x,
            area.y,
            &display,
            theme.placeholder,
            theme.background,
        );
        return;
    }

    // Get image data
    let data = match &node.source {
        Some(ImageSource::Data(bytes)) => bytes.clone(),
        Some(ImageSource::File(path)) => {
            // Load file at render time
            match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(_) => {
                    let error_msg = format!("[Image: failed to load {}]", path);
                    let display: String = error_msg.chars().take(area.width as usize).collect();
                    _buffer.write_str(area.x, area.y, &display, theme.error, theme.background);
                    return;
                }
            }
        }
        None => {
            // No source - render placeholder
            let placeholder = "[Image: no source]";
            let display: String = placeholder.chars().take(area.width as usize).collect();
            _buffer.write_str(
                area.x,
                area.y,
                &display,
                theme.placeholder,
                theme.background,
            );
            return;
        }
    };

    // Queue the image for rendering via Kitty protocol
    ctx.queue_image(PendingImage {
        cell_x: area.x,
        cell_y: area.y,
        data,
        id: node.id,
        cell_width: node.cell_width.unwrap_or(area.width),
        cell_height: node.cell_height.unwrap_or(area.height),
    });

    // Fill the character buffer area with spaces (image will overlay via Kitty)
    for dy in 0..area.height {
        let row_str = " ".repeat(area.width as usize);
        _buffer.write_str(
            area.x,
            area.y + dy,
            &row_str,
            theme.foreground,
            theme.background,
        );
    }
}

fn render_terminal(buffer: &mut Buffer, node: &TerminalNode, area: Rect, ctx: &mut RenderContext) {
    let is_focused = ctx.is_next_focused();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = current_theme();

    // Calculate content area (accounting for border)
    let (content_x, content_y, content_width, content_height) = if node.border {
        if area.width < 2 || area.height < 2 {
            return; // Too small for border
        }

        // Draw border
        let border_fg = if is_focused {
            theme.primary
        } else {
            theme.muted
        };

        // Top border
        buffer.set(area.x, area.y, '┌', border_fg, theme.background);
        for x in (area.x + 1)..(area.x + area.width - 1) {
            buffer.set(x, area.y, '─', border_fg, theme.background);
        }
        buffer.set(
            area.x + area.width - 1,
            area.y,
            '┐',
            border_fg,
            theme.background,
        );

        // Title in top border
        if let Some(ref title) = node.title {
            let title_str = format!(" {} ", title);
            let title_x = area.x + 2;
            buffer.write_str(title_x, area.y, &title_str, border_fg, theme.background);
        }

        // Side borders
        for y in (area.y + 1)..(area.y + area.height - 1) {
            buffer.set(area.x, y, '│', border_fg, theme.background);
            buffer.set(
                area.x + area.width - 1,
                y,
                '│',
                border_fg,
                theme.background,
            );
        }

        // Bottom border
        buffer.set(
            area.x,
            area.y + area.height - 1,
            '└',
            border_fg,
            theme.background,
        );
        for x in (area.x + 1)..(area.x + area.width - 1) {
            buffer.set(
                x,
                area.y + area.height - 1,
                '─',
                border_fg,
                theme.background,
            );
        }
        buffer.set(
            area.x + area.width - 1,
            area.y + area.height - 1,
            '┘',
            border_fg,
            theme.background,
        );

        (
            area.x + 1,
            area.y + 1,
            area.width - 2,
            area.height - 2,
        )
    } else {
        (area.x, area.y, area.width, area.height)
    };

    if content_width == 0 || content_height == 0 {
        return;
    }

    // Get terminal buffer
    let term_buffer = node.handle.get_buffer();
    let term_rows = term_buffer.rows().min(content_height as usize);
    let term_cols = term_buffer.cols().min(content_width as usize);

    // Copy terminal buffer cells to screen buffer
    for row in 0..term_rows {
        for col in 0..term_cols {
            if let Some(cell) = term_buffer.get_cell(row, col) {
                let screen_x = content_x + col as u16;
                let screen_y = content_y + row as u16;

                // Skip wide continuation cells
                if cell.wide_continuation {
                    continue;
                }

                buffer.set_cell(screen_x, screen_y, *cell);
            }
        }
    }

    // Draw cursor if terminal is focused and cursor is visible
    if is_focused && term_buffer.cursor_visible() {
        let cursor_row = term_buffer.cursor_row();
        let cursor_col = term_buffer.cursor_col();

        if cursor_row < term_rows && cursor_col < term_cols {
            let cursor_x = content_x + cursor_col as u16;
            let cursor_y = content_y + cursor_row as u16;

            // Get the cell at cursor position to invert it
            if let Some(cell) = term_buffer.get_cell(cursor_row, cursor_col) {
                // Invert foreground and background for cursor
                let inverted_cell = crate::buffer::Cell::styled(
                    cell.ch,
                    cell.bg, // Swap fg/bg
                    cell.fg,
                    cell.bold,
                    cell.italic,
                    cell.underline,
                    cell.dim,
                );
                buffer.set_cell(cursor_x, cursor_y, inverted_cell);
            }
        }
    }
}
