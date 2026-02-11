// Test file to verify view! macro works with all Phase 1 features
// This should compile and produce identical output to builder pattern

use telex::prelude::*;
use telex::Color;

/// Test that JSX syntax compiles for all widget types
#[allow(dead_code)]
fn test_jsx_syntax(cx: Scope) -> View {
    // State for interactive widgets
    let count = state!(cx, || 0);
    let text_value = state!(cx, String::new);
    let textarea_value = state!(cx, String::new);
    let selected = state!(cx, || 0);
    let checked = state!(cx, || false);
    let show_modal = state!(cx, || false);

    // Clones for closures
    let c1 = count.clone();
    let c2 = count.clone();
    let tv = text_value.clone();
    let tav = textarea_value.clone();
    let sel = selected.clone();
    let chk = checked.clone();
    let sm = show_modal.clone();
    let sm2 = show_modal.clone();
    let sm3 = show_modal.clone();

    view! {
        <VStack spacing={1}>
            // Basic text
            <Text>"Hello, JSX!"</Text>

            // Expression in text
            <Text>{format!("Count: {}", count.get())}</Text>

            // Styled text
            <StyledText bold={true} color={Color::Cyan}>"Bold cyan text"</StyledText>

            // HStack with buttons
            <HStack spacing={2}>
                <Button on_press={move || c1.update(|n| *n -= 1)}>"-"</Button>
                <Button on_press={move || c2.update(|n| *n += 1)}>"+"</Button>
            </HStack>

            // Box with border, flex, and min_height constraint
            <Box border={true} flex={1} min_height={5}>
                <VStack>
                    <Text>"Inside a box"</Text>

                    // TextInput
                    <TextInput
                        value={text_value.get()}
                        placeholder={"Type here..."}
                        on_change={move |s| tv.set(s)}
                    />

                    // List
                    <List
                        items={vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string()]}
                        selected={selected.get()}
                        on_select={move |i| sel.set(i)}
                    />

                    // TextArea
                    <TextArea
                        value={textarea_value.get()}
                        placeholder={"Multi-line input..."}
                        rows={3}
                        on_change={move |s| tav.set(s)}
                    />

                    // Checkbox
                    <Checkbox
                        checked={checked.get()}
                        on_toggle={move |v| chk.set(v)}
                    >"Enable feature"</Checkbox>

                    // Button to show modal
                    <Button on_press={move || sm.set(true)}>"Show Modal"</Button>
                </VStack>
            </Box>

            // Spacer
            <Spacer />

            // Modal
            <Modal
                visible={show_modal.get()}
                title={"Test Modal"}
                width={50}
                height={30}
                on_dismiss={move || sm2.set(false)}
            >
                <VStack>
                    <Text>"Modal content"</Text>
                    <Button on_press={move || sm3.set(false)}>"Close"</Button>
                </VStack>
            </Modal>
        </VStack>
    }
}
