#!/bin/bash
# examples.sh - Usage: ./examples.sh [list|run <name>|all] or no args for interactive menu

set -e

EXAMPLES_DIR="crates/telex/examples"
APPS_DIR="examples"

# Get sorted list of single-file examples
get_examples() {
    ls "$EXAMPLES_DIR"/*.rs 2>/dev/null | xargs -n1 basename | sed 's/\.rs$//' | sort
}

# Get list of app examples (packages)
get_apps() {
    ls -d "$APPS_DIR"/*/ 2>/dev/null | xargs -n1 basename | sort
}

# Run a single example (handles both single-file and app packages)
run_example() {
    local name="$1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Running: $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Check if it's an app package
    if [ -d "$APPS_DIR/$name" ]; then
        cargo run -p "$name"
    else
        cargo run -p telex-tui --example "$name"
    fi
}

# List all examples
list_examples() {
    echo "Examples:"
    echo ""
    get_examples | while read -r ex; do
        # Extract description from first line comment
        desc=$(head -1 "$EXAMPLES_DIR/$ex.rs" | sed 's|^//! ||' | sed 's|^//||')

        # Check if experimental and add marker
        if is_experimental "$ex"; then
            if [ "$ex" = "29_canvas" ]; then
                echo ""
                echo "  ──────────────────────────────"
                echo "  ⚠️  Experimental (in development):"
                echo ""
            fi
            printf "  ⚠️  %-23s %s\n" "$ex" "$desc"
        else
            printf "  %-25s %s\n" "$ex" "$desc"
        fi
    done

    echo ""
    echo "Apps:"
    echo ""
    get_apps | while read -r app; do
        # Extract description from Cargo.toml
        desc=$(grep '^description' "$APPS_DIR/$app/Cargo.toml" 2>/dev/null | sed 's/description = "//' | sed 's/"$//' || echo "")
        printf "  %-25s %s\n" "$app" "$desc"
    done
}

# Check if example is experimental (29-33)
is_experimental() {
    local name="$1"
    case "$name" in
        29_*|30_*|31_*|32_*|33_*) return 0 ;;
        *) return 1 ;;
    esac
}

# Interactive menu with scrolling
interactive_menu() {
    local examples=($(get_examples))
    local apps=($(get_apps))

    # Split examples into stable and experimental
    local stable_examples=()
    local experimental_examples=()
    for ex in "${examples[@]}"; do
        if is_experimental "$ex"; then
            experimental_examples+=("$ex")
        else
            stable_examples+=("$ex")
        fi
    done

    # Build items list with separators
    local all_items=("${stable_examples[@]}" "---experimental---" "${experimental_examples[@]}" "---apps---" "${apps[@]}")
    local count=${#all_items[@]}
    local current=0

    # Get terminal height and calculate visible window
    local term_height=$(tput lines)
    local header_lines=2  # For title and instructions
    local footer_lines=1  # For status line
    local visible_lines=$((term_height - header_lines - footer_lines))
    local window_start=0

    while true; do
        clear

        # Header
        echo "Telex Examples (↑/↓ or j/k: Navigate | Enter: Run | q/Ctrl-Q: Quit)"
        echo ""

        # Reserve space for indicators (2 lines if needed)
        local actual_visible=$((visible_lines - 2))

        # Calculate window position to keep cursor visible
        if [ $current -lt $window_start ]; then
            window_start=$current
        elif [ $current -ge $((window_start + actual_visible)) ]; then
            window_start=$((current - actual_visible + 1))
        fi

        local window_end=$((window_start + actual_visible))

        # Show "more above" indicator
        if [ $window_start -gt 0 ]; then
            echo "  ↑ More above..."
        fi

        # Display visible items
        for ((i=window_start; i<window_end && i<count; i++)); do
            local item="${all_items[$i]}"
            if [ "$item" = "---experimental---" ]; then
                echo "  ──────────────────────────────"
                echo "  ⚠️  Experimental (in development):"
            elif [ "$item" = "---apps---" ]; then
                echo "  ──────────────────────────────"
                echo "  Apps:"
            elif [ "$i" -eq "$current" ]; then
                if is_experimental "$item"; then
                    echo "  ▶ ⚠️  $item"
                else
                    echo "  ▶ $item"
                fi
            else
                if is_experimental "$item"; then
                    echo "    ⚠️  $item"
                else
                    echo "    $item"
                fi
            fi
        done

        # Show "more below" indicator
        if [ $window_end -lt $count ]; then
            echo "  ↓ More below..."
        fi

        # Read single keypress (disable flow control so Ctrl-Q works)
        old_stty=$(stty -g)
        stty -ixon
        read -rsn1 key
        stty "$old_stty"

        case "$key" in
            $'\x11')  # Ctrl-Q
                echo ""
                echo "Goodbye!"
                exit 0
                ;;
            k|A)  # up arrow or k
                ((current > 0)) && ((current--))
                # Skip separators
                while [ "${all_items[$current]}" = "---experimental---" ] || [ "${all_items[$current]}" = "---apps---" ]; do
                    ((current > 0)) && ((current--)) || break
                done
                ;;
            j|B)  # down arrow or j
                ((current < count - 1)) && ((current++))
                # Skip separators
                while [ "${all_items[$current]}" = "---experimental---" ] || [ "${all_items[$current]}" = "---apps---" ]; do
                    ((current < count - 1)) && ((current++)) || break
                done
                ;;
            q)
                echo ""
                echo "Goodbye!"
                exit 0
                ;;
            "")  # Enter
                local selected="${all_items[$current]}"
                if [ "$selected" != "---experimental---" ] && [ "$selected" != "---apps---" ]; then
                    run_example "$selected"
                    echo ""
                    echo "Press any key to continue (or Ctrl-Q to quit)..."
                    # Disable flow control so Ctrl-Q isn't intercepted
                    old_stty=$(stty -g)
                    stty -ixon
                    read -rsn1 continue_key
                    stty "$old_stty"
                    # Ctrl-Q is ASCII 17
                    if [[ "$continue_key" == $'\x11' ]]; then
                        echo ""
                        echo "Goodbye!"
                        exit 0
                    fi
                fi
                ;;
        esac
    done
}

# Run all examples in sequence
run_all() {
    echo "Running single-file examples..."
    get_examples | while read -r ex; do
        run_example "$ex"
        echo ""
            done

    echo ""
    echo "Running app examples..."
    get_apps | while read -r app; do
        run_example "$app"
        echo ""
            done
    echo "All examples completed!"
}

# Main
case "${1:-}" in
    list)
        list_examples
        ;;
    run)
        if [ -z "${2:-}" ]; then
            echo "Usage: $0 run <example_number_or_name>"
            echo "Example: $0 run 02"
            echo "Example: $0 run chat"
            exit 1
        fi
        # Check if it's an app first
        if [ -d "$APPS_DIR/${2}" ]; then
            run_example "${2}"
        else
            # Find matching example
            match=$(get_examples | grep -E "^0*${2}" | head -1)
            if [ -z "$match" ]; then
                echo "No example matching '$2' found"
                exit 1
            fi
            run_example "$match"
        fi
        ;;
    all)
        run_all
        ;;
    ""|menu)
        interactive_menu
        ;;
    *)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  (none)    Interactive menu"
        echo "  list      List all examples"
        echo "  run <n>   Run example by number (e.g., run 02)"
        echo "  all       Run all examples in sequence"
        exit 1
        ;;
esac
