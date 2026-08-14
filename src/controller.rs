//! ViewController — manages a single screen of content.
//!
//! `ViewController` is similar to Apple's `UIViewController`. It manages
//! a view hierarchy and provides lifecycle methods.
//!
//! ```rust,no_run
//! use uikit::prelude::*;
//!
//! struct MainViewController;
//!
//! impl ViewControllerDelegate for MainViewController {
//!     fn load_view(&mut self) -> View {
//!         let mut root = View::empty();
//!         root.set_frame(0.0, 0.0, 800.0, 600.0);
//!
//!         let label = View::new(Label::new("Hello, TontooOS!"))
//!             .with_frame(16.0, 16.0, 200.0, 30.0);
//!         root.add_subview(label);
//!
//!         root
//!     }
//! }
//!
//! let mut vc = ViewController::new(MainViewController);
//! vc.load_view_if_needed();
//! ```

use crate::view::View;
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════
// ViewControllerDelegate
// ═══════════════════════════════════════════════════════════════

/// Delegate trait for view controllers.
///
/// Implement this to customize the view controller's behavior.
pub trait ViewControllerDelegate: Send + 'static {
    /// Load and return the view for this controller.
    fn load_view(&mut self) -> View;

    /// Called after the view has been loaded.
    fn view_did_load(&mut self) {}

    /// Called when the view is about to appear.
    fn view_will_appear(&mut self) {}

    /// Called when the view has appeared.
    fn view_did_appear(&mut self) {}

    /// Called when the view is about to disappear.
    fn view_will_disappear(&mut self) {}

    /// Called when the view has disappeared.
    fn view_did_disappear(&mut self) {}

    /// Called when the view receives a memory warning.
    fn did_receive_memory_warning(&mut self) {}
}

// ═══════════════════════════════════════════════════════════════
// ViewController
// ═══════════════════════════════════════════════════════════════

/// A view controller manages a single screen of content.
///
/// View controllers are the core building block for app navigation.
/// Each screen in your app is typically managed by a view controller.
pub struct ViewController {
    title: String,
    view: Option<View>,
    delegate: Box<dyn ViewControllerDelegate>,
    presented: Option<Box<ViewController>>,
    presenting: Option<Arc<Mutex<ViewController>>>,
    is_view_loaded: bool,
}

impl ViewController {
    /// Create a new view controller with the given delegate.
    pub fn new(delegate: impl ViewControllerDelegate) -> Self {
        Self {
            title: String::new(),
            view: None,
            delegate: Box::new(delegate),
            presented: None,
            presenting: None,
            is_view_loaded: false,
        }
    }

    /// Create a view controller with a title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Get the title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Get the view (loads if needed).
    pub fn view(&mut self) -> &View {
        self.load_view_if_needed();
        self.view.as_ref().unwrap()
    }

    /// Get a mutable reference to the view (loads if needed).
    pub fn view_mut(&mut self) -> &mut View {
        self.load_view_if_needed();
        self.view.as_mut().unwrap()
    }

    /// Check if the view has been loaded.
    pub fn is_view_loaded(&self) -> bool {
        self.is_view_loaded
    }

    /// Load the view if it hasn't been loaded yet.
    pub fn load_view_if_needed(&mut self) {
        if !self.is_view_loaded {
            self.load_view();
        }
    }

    /// Force load the view.
    fn load_view(&mut self) {
        self.view = Some(self.delegate.load_view());
        self.is_view_loaded = true;
        self.delegate.view_did_load();
    }

    /// Unload the view (free memory).
    pub fn unload_view(&mut self) {
        if self.is_view_loaded {
            self.delegate.view_will_disappear();
            self.view = None;
            self.is_view_loaded = false;
        }
    }

    // ─── Presentation ──────────────────────────────────────

    /// Present another view controller modally.
    pub fn present(&mut self, vc: ViewController, animated: bool) {
        if animated {
            // TODO: Add presentation animation
        }
        self.delegate.view_will_disappear();
        self.presented = Some(Box::new(vc));
    }

    /// Dismiss the presented view controller.
    pub fn dismiss(&mut self, animated: bool) {
        if let Some(mut vc) = self.presented.take() {
            if animated {
                // TODO: Add dismissal animation
            }
            vc.delegate.view_will_disappear();
            self.delegate.view_did_appear();
        }
    }

    /// Get the presented view controller, if any.
    pub fn presented(&self) -> Option<&ViewController> {
        self.presented.as_ref().map(|vc| vc.as_ref())
    }

    /// Get the presenting view controller, if any.
    pub fn presenting(&self) -> Option<Arc<Mutex<ViewController>>> {
        self.presenting.clone()
    }

    // ─── View Lifecycle ────────────────────────────────────

    /// Notify the controller that the view will appear.
    pub fn notify_will_appear(&mut self) {
        self.delegate.view_will_appear();
        if let Some(ref mut view) = self.view {
            for _subview in view.subviews_mut() {
                // Subviews can also respond to lifecycle events
            }
        }
    }

    /// Notify the controller that the view did appear.
    pub fn notify_did_appear(&mut self) {
        self.delegate.view_did_appear();
    }

    /// Notify the controller that the view will disappear.
    pub fn notify_will_disappear(&mut self) {
        self.delegate.view_will_disappear();
    }

    /// Notify the controller that the view did disappear.
    pub fn notify_did_disappear(&mut self) {
        self.delegate.view_did_disappear();
    }

    /// Notify the controller of a memory warning.
    pub fn notify_memory_warning(&mut self) {
        self.delegate.did_receive_memory_warning();
    }

    // ─── Navigation ────────────────────────────────────────

    /// Navigate to another view controller (push).
    pub fn push(&mut self, _vc: ViewController, animated: bool) {
        if animated {
            // TODO: Add push animation
        }
        // In a real UIKit, this would be handled by a navigation controller
    }

    /// Pop the top view controller (pop).
    pub fn pop(&mut self, animated: bool) -> Option<ViewController> {
        if animated {
            // TODO: Add pop animation
        }
        // In a real UIKit, this would be handled by a navigation controller
        None
    }

    // ─── Rendering ─────────────────────────────────────────

    /// Render the view controller's view to a GTK4 widget.
    pub fn to_gtk(&mut self) -> gtk::Widget {
        self.load_view_if_needed();
        self.view.as_ref().unwrap().to_gtk()
    }
}

// ═══════════════════════════════════════════════════════════════
// NavigationController
// ═══════════════════════════════════════════════════════════════

/// A navigation controller manages a stack of view controllers.
///
/// Similar to Apple's `UINavigationController`, it provides a stack-based
/// navigation model with push/pop transitions.
pub struct NavigationController {
    stack: Vec<ViewController>,
    root: Option<ViewController>,
}

impl NavigationController {
    /// Create a navigation controller with a root view controller.
    pub fn new(root: ViewController) -> Self {
        Self {
            stack: Vec::new(),
            root: Some(root),
        }
    }

    /// Get the top view controller.
    pub fn top(&self) -> Option<&ViewController> {
        self.stack.last().or(self.root.as_ref())
    }

    /// Get a mutable reference to the top view controller.
    pub fn top_mut(&mut self) -> Option<&mut ViewController> {
        self.stack.last_mut().or(self.root.as_mut())
    }

    /// Get the root view controller.
    pub fn root(&self) -> Option<&ViewController> {
        self.root.as_ref()
    }

    /// Push a view controller onto the stack.
    pub fn push(&mut self, vc: ViewController, _animated: bool) {
        if let Some(top) = self.stack.last_mut() {
            top.notify_will_disappear();
        }
        self.stack.push(vc);
        if let Some(top) = self.stack.last_mut() {
            top.notify_will_appear();
        }
    }

    /// Pop the top view controller from the stack.
    pub fn pop(&mut self, _animated: bool) -> Option<ViewController> {
        if let Some(mut vc) = self.stack.pop() {
            vc.notify_will_disappear();
            if let Some(top) = self.stack.last_mut() {
                top.notify_will_appear();
            }
            Some(vc)
        } else {
            None
        }
    }

    /// Pop to the root view controller.
    pub fn pop_to_root(&mut self, animated: bool) {
        while self.stack.len() > 1 {
            self.pop(animated);
        }
    }

    /// Get the view stack depth.
    pub fn depth(&self) -> usize {
        self.stack.len() + if self.root.is_some() { 1 } else { 0 }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::View;

    struct TestDelegate;

    impl ViewControllerDelegate for TestDelegate {
        fn load_view(&mut self) -> View {
            View::empty().with_frame(0.0, 0.0, 400.0, 300.0)
        }
    }

    #[test]
    fn view_controller_creation() {
        let mut vc = ViewController::new(TestDelegate);
        assert!(!vc.is_view_loaded());
        assert_eq!(vc.title(), "");

        let _view = vc.view();
        assert!(vc.is_view_loaded());
    }

    #[test]
    fn navigation_controller() {
        let root = ViewController::new(TestDelegate);
        let mut nav = NavigationController::new(root);

        assert_eq!(nav.depth(), 1);

        let vc2 = ViewController::new(TestDelegate).with_title("Second");
        nav.push(vc2, false);
        assert_eq!(nav.depth(), 2);
        assert_eq!(nav.top().unwrap().title(), "Second");

        nav.pop(false);
        assert_eq!(nav.depth(), 1);
    }
}
