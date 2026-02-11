//! Context system for sharing state without prop drilling.
//!
//! Allows parent components to provide values that child components can access
//! without explicitly passing them through every level of the component tree.
//!
//! # Example
//! ```rust,ignore
//! use telex::prelude::*;
//!
//! #[derive(Clone)]
//! struct AppTheme {
//!     accent_color: Color,
//! }
//!
//! fn App(cx: Scope) -> View {
//!     // Provide theme context for all children
//!     cx.provide_context(AppTheme {
//!         accent_color: Color::Cyan,
//!     });
//!
//!     view! {
//!         <VStack>
//!             <Header />
//!             <Content />
//!         </VStack>
//!     }
//! }
//!
//! fn Header(cx: Scope) -> View {
//!     // Access theme without prop drilling
//!     let theme = cx.use_context::<AppTheme>().unwrap();
//!     view! { <Text color={theme.accent_color}>"Header"</Text> }
//! }
//! ```

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Storage for context values, keyed by type.
#[derive(Default)]
pub struct ContextStorage {
    values: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
}

impl ContextStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Provide a value in the context.
    /// If a value of this type already exists, it will be replaced.
    pub fn provide<T: Clone + 'static>(&self, value: T) {
        let type_id = TypeId::of::<T>();
        self.values.borrow_mut().insert(type_id, Rc::new(value));
    }

    /// Get a value from the context.
    /// Returns None if no value of this type has been provided.
    pub fn get<T: Clone + 'static>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.values
            .borrow()
            .get(&type_id)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    /// Check if a value of this type exists in the context.
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.values.borrow().contains_key(&type_id)
    }

    /// Clear all context values.
    pub fn clear(&self) {
        self.values.borrow_mut().clear();
    }
}
