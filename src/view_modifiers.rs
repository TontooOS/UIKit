//! View modifiers — SwiftUI-style modifiers backed by core View.
//!
//! These modifiers are used by `TontooUI/views` category. They attach
//! metadata to the core `View` so the higher-level declarative layer can
//! present pickers, sheets, swipe actions, backgrounds and glass effects
//! without branching in `View::to_gtk`. Rendering remains in `ViewContent`
//! / `ShaderView`; this module only provides the declarative state.

use crate::style::Color;

/// Swipe side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeEdge { Leading, Trailing }

/// A single swipe action (label + role).
#[derive(Debug, Clone)]
pub struct SwipeAction {
    pub label: String,
    pub tint: Option<Color>,
    pub is_destructive: bool,
}

/// Control size — mirrors `SwiftUI.ControlSize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSize { Mini, Small, Regular, Large, ExtraLarge }

impl Default for ControlSize { fn default() -> Self { Self::Regular } }

/// Background container for NavigationSplitView / NavigationStack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerBackground { Default, Blue, Custom }

/// Core modifiers attached to a `View`.
///
/// `View` stores an optional `ViewModifiers`. Higher-level TontooUI
/// elements (e.g. `MusicPicker`, `GlassEffect`) set these flags via
/// `View::with_modifier`. The renderer / preview layer reads them to
/// decide overlay, sheet, swipe or glass presentation. For the palette
/// previews (directly on window, no extra card) the flags only affect
/// the returned `gtk::Widget` in the TontooUI layer — this core layer
/// stays lightweight.
#[derive(Debug, Clone, Default)]
pub struct ViewModifiers {
    pub music_picker_presented: bool,
    pub app_store_overlay_presented: bool,
    pub app_store_overlay_app_id: Option<String>,
    pub manage_subscriptions_presented: bool,
    pub swipe_container_single_active: bool,
    pub swipe_actions: Vec<(SwipeEdge, SwipeAction)>,
    pub swipe_state_active: bool,
    pub navigation_split_background: Option<ContainerBackground>,
    pub navigation_container_background: Option<Color>,
    pub control_size: ControlSize,
    pub background_extension_enabled: bool,
    pub background_extension_duplicate: bool,
    pub glass_effect_enabled: bool,
    pub glass_tint: Option<Color>,
}

// ── View extension trait ─────────────────────────────────────────────

/// Declarative modifiers — chainable, each consumes `self` and returns `Self`.
/// Implemented for `crate::view::View` in this file.
pub trait ViewModifierExt {
    fn musicPicker(self, presented: bool) -> Self;
    fn appStoreOverlay(self, presented: bool, app_id: impl Into<String>) -> Self;
    fn manageSubscriptionsSheet(self, presented: bool) -> Self;
    fn swipeContainer(self, single_active: bool) -> Self;
    fn swipeAction(self, edge: SwipeEdge, label: impl Into<String>, destructive: bool) -> Self;
    fn swipeState(self, active: bool) -> Self;
    fn navigationSplitViewBackground(self, style: ContainerBackground) -> Self;
    fn navigationContainerBackground(self, color: Color) -> Self;
    fn controlSize(self, size: ControlSize) -> Self;
    fn backgroundExtensionEffect(self, enabled: bool) -> Self;
    fn glassEffect(self, tint: Option<Color>) -> Self;
    fn glassEffectTined(self, color: Color) -> Self;
}

impl ViewModifierExt for crate::view::View {
    fn musicPicker(self, presented: bool) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().music_picker_presented = presented;
        self
    }
    fn appStoreOverlay(self, presented: bool, app_id: impl Into<String>) -> Self {
        let id = self.id();
        let mut map = mods_map().lock().unwrap();
        let m = map.entry(id).or_default();
        m.app_store_overlay_presented = presented;
        m.app_store_overlay_app_id = Some(app_id.into());
        self
    }
    fn manageSubscriptionsSheet(self, presented: bool) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().manage_subscriptions_presented = presented;
        self
    }
    fn swipeContainer(self, single_active: bool) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().swipe_container_single_active = single_active;
        self
    }
    fn swipeAction(self, edge: SwipeEdge, label: impl Into<String>, destructive: bool) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().swipe_actions.push((edge, SwipeAction { label: label.into(), tint: None, is_destructive: destructive }));
        self
    }
    fn swipeState(self, active: bool) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().swipe_state_active = active;
        self
    }
    fn navigationSplitViewBackground(self, style: ContainerBackground) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().navigation_split_background = Some(style);
        self
    }
    fn navigationContainerBackground(self, color: Color) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().navigation_container_background = Some(color);
        self
    }
    fn controlSize(self, size: ControlSize) -> Self {
        let id = self.id();
        mods_map().lock().unwrap().entry(id).or_default().control_size = size;
        self
    }
    fn backgroundExtensionEffect(self, enabled: bool) -> Self {
        let id = self.id();
        let mut map = mods_map().lock().unwrap();
        let m = map.entry(id).or_default();
        m.background_extension_enabled = enabled;
        m.background_extension_duplicate = enabled;
        self
    }
    fn glassEffect(self, tint: Option<Color>) -> Self {
        let id = self.id();
        let mut map = mods_map().lock().unwrap();
        let m = map.entry(id).or_default();
        m.glass_effect_enabled = true;
        m.glass_tint = tint;
        self
    }
    fn glassEffectTined(self, color: Color) -> Self { self.glassEffect(Some(color)) }
}

// ── storage on View ─────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::{OnceLock, Mutex};

// Side store because `View` is already defined in view.rs and we avoid
// changing its layout for ABI stability. Modifiers are keyed by ViewId.
static MODS: OnceLock<Mutex<HashMap<crate::view::ViewId, ViewModifiers>>> = OnceLock::new();
fn mods_map() -> &'static Mutex<HashMap<crate::view::ViewId, ViewModifiers>> {
    MODS.get_or_init(|| Mutex::new(HashMap::new()))
}

trait ViewModifiersStore {
    fn view_modifiers(&self) -> ViewModifiers;
}

impl ViewModifiersStore for crate::view::View {
    fn view_modifiers(&self) -> ViewModifiers {
        let map = mods_map().lock().unwrap();
        map.get(&self.id()).cloned().unwrap_or_default()
    }
}

// Public helpers for the TontooUI preview layer
pub fn get_modifiers(view_id: crate::view::ViewId) -> ViewModifiers {
    mods_map().lock().unwrap().get(&view_id).cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::View;
    use crate::style::Color;
    #[test]
    fn control_size_default() { assert_eq!(ControlSize::default(), ControlSize::Regular); }
    #[test]
    fn modifiers_chain() {
        let v = View::empty().musicPicker(true).controlSize(ControlSize::Large).backgroundExtensionEffect(true);
        let m = v.view_modifiers();
        assert!(m.music_picker_presented);
        assert_eq!(m.control_size, ControlSize::Large);
        assert!(m.background_extension_enabled);
    }
    #[test]
    fn glass_modifier() {
        let v = View::empty().glassEffect(Some(Color::RED));
        assert!(v.view_modifiers().glass_effect_enabled);
    }
    #[test]
    fn swipe_modifiers() {
        let v = View::empty().swipeContainer(true).swipeAction(SwipeEdge::Trailing, "Delete", true).swipeState(true);
        let m = v.view_modifiers();
        assert!(m.swipe_container_single_active);
        assert_eq!(m.swipe_actions.len(), 1);
        assert!(m.swipe_state_active);
    }
}
