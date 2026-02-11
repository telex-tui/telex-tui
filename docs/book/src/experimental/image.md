# Image Display

> ⚠️ **Experimental Feature**
> Image widget requires the Kitty graphics protocol and only works in compatible terminals (Kitty, Ghostty, WezTerm).
> Other terminals will show a placeholder message.

Display PNG, JPEG, and GIF images using the Kitty graphics protocol.

```rust
// From file
View::image()
    .file("logo.png")
    .width(40)   // cells
    .height(20)  // cells
    .build()

// From bytes
View::image()
    .data(include_bytes!("logo.png"))
    .alt("Company logo")
    .build()
```

Run with: `cargo run -p telex-tui --example 30_image`

## Terminal Compatibility

**Supported terminals:**
- Kitty
- Ghostty
- WezTerm

**Unsupported terminals:**
Other terminals will display the alt text or a placeholder message: `[Image: requires Kitty/Ghostty/WezTerm]`

## Features

- **Format support:** PNG, JPEG, GIF
- **Animations:** GIF animations handled natively by Kitty
- **Auto-sizing:** Images auto-detect dimensions if not specified
- **Caching:** Uses Kitty's image ID system for efficient re-renders

## Image Sources

### From File Path
```rust
View::image()
    .file("screenshots/demo.png")
    .build()
```

### From Bytes
```rust
const LOGO: &[u8] = include_bytes!("../assets/logo.png");

View::image()
    .data(LOGO)
    .build()
```

### With Sizing
```rust
View::image()
    .file("banner.png")
    .width(60)   // Force 60 columns wide
    .height(15)  // Force 15 rows tall
    .build()
```

## Accessibility

Use `.alt()` to provide fallback text:
```rust
View::image()
    .file("chart.png")
    .alt("Sales chart showing 40% growth in Q4")
    .build()
```

## Limitations

- Requires Kitty graphics protocol
- No image manipulation (resize, crop, filters)
- No lazy loading
- Images are re-encoded on every render (cached by Kitty)

## How It Works

The image widget:
1. Encodes image data as base64
2. Generates Kitty graphics protocol escape sequences
3. Writes sequences to terminal
4. Kitty decodes and displays the image as pixel overlay
5. Terminal cell buffer shows spaces (image overlays them)

## Future Improvements

- Fallback rendering for non-Kitty terminals (ASCII art, sixel)
- Image caching to avoid re-encoding
- Lazy loading for large images
- Basic transformations (scale, crop)
