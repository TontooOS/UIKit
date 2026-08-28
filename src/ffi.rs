//! C FFI exports for UIKit.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

fn read_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok().map(str::to_owned)
}

fn set_error(error_out: *mut *mut c_char, message: &str) {
    if error_out.is_null() {
        return;
    }
    if let Ok(c) = CString::new(message.to_owned()) {
        unsafe { *error_out = c.into_raw() };
    }
}

/// One-time GTK initialization for library consumers that have no toolkit
/// of their own. Safe to call repeatedly.
pub(crate) fn gtk_init_guard() -> Result<(), String> {
    if gtk::is_initialized() {
        Ok(())
    } else {
        gtk::init().map_err(|e| format!("gtk init failed: {e}"))
    }
}

/// The framework version as a static C string.
#[no_mangle]
pub extern "C" fn tontoo_uikit_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Load global CSS that applies to all GTK windows of the process.
/// Returns 0 on success, -1 on null input, -2 on GTK init failure.
///
/// # Safety
///
/// `css` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_uikit_load_css(css: *const c_char) -> c_int {
    let Some(css) = read_str(css) else {
        return -1;
    };
    if gtk_init_guard().is_err() {
        return -2;
    }
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        0
    } else {
        -3
    }
}

/// Apply CSS to a single widget.
///
/// Returns 0 on success, -1 on null arguments, -2 on GTK init failure.
///
/// # Safety
///
/// `widget` must be a valid `GtkWidget*` and `css` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_uikit_widget_apply_css(
    widget: *mut gtk::ffi::GtkWidget,
    css: *const c_char,
) -> c_int {
    if widget.is_null() {
        return -1;
    }
    let Some(css) = read_str(css) else {
        return -1;
    };
    if gtk_init_guard().is_err() {
        return -2;
    }
    let borrowed = unsafe {
        <gtk::Widget as glib::translate::FromGlibPtrBorrow<*mut gtk::ffi::GtkWidget>>::from_glib_borrow(widget)
    };
    crate::apply_css(&*borrowed, &css);
    0
}

/// Free a string returned by this library.
///
/// # Safety
///
/// `s` must be a pointer returned by this API or null.
#[no_mangle]
pub unsafe extern "C" fn tontoo_uikit_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
