//! ShaderView widget — renders GLSL shaders using GTK4's GLArea.

use crate::shader::{Shader, Uniforms};
use crate::style::{Padding, Rect, Size};
use crate::view::{View, ViewContent};
use crate::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, GLArea};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ShaderView {
    id: WidgetId,
    shader: Shader,
    uniforms: Uniforms,
    auto_animate: bool,
    position_mode: PositionMode,
    position: Position,
}

impl ShaderView {
    pub fn new(shader: Shader) -> Self {
        Self {
            id: next_widget_id(),
            shader,
            uniforms: Uniforms::new(),
            auto_animate: true,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position_mode = PositionMode::Absolute;
        self.position.x = Some(x);
        self.position.y = Some(y);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.position.width = Some(width);
        self.position.height = Some(height);
        self
    }

    pub fn uniforms(mut self, uniforms: Uniforms) -> Self {
        self.uniforms = uniforms;
        self
    }

    pub fn auto_animate(mut self, auto: bool) -> Self {
        self.auto_animate = auto;
        self
    }

    pub fn to_view(self, x: f32, y: f32, width: f32, height: f32) -> View {
        View::new(self).with_frame(x, y, width, height)
    }
}

impl Default for ShaderView {
    fn default() -> Self {
        Self::new(Shader::gradient())
    }
}

impl ViewContent for ShaderView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let gl_area = GLArea::new();
        gl_area.set_hexpand(true);
        gl_area.set_vexpand(true);
        gl_area.set_required_version(3, 0);
        gl_area.set_auto_render(true);

        if frame.width > 0.0 {
            gl_area.set_width_request(frame.width as i32);
        }
        if frame.height > 0.0 {
            gl_area.set_height_request(frame.height as i32);
        }

        let shader = self.shader.clone();
        let uniforms = Rc::new(RefCell::new(self.uniforms));
        let auto_animate = self.auto_animate;
        let start_time = std::time::Instant::now();

        let mouse = Rc::new(RefCell::new((0.0_f32, 0.0_f32)));
        {
            let m1 = mouse.clone();
            let motion = gtk::EventControllerMotion::new();
            motion.connect_enter(move |_, x, y| {
                *m1.borrow_mut() = (x as f32, y as f32);
            });
            let m2 = mouse.clone();
            motion.connect_motion(move |_, x, y| {
                *m2.borrow_mut() = (x as f32, y as f32);
            });
            gl_area.add_controller(motion);
        }

        let uniforms_clone = uniforms.clone();
        let mouse_clone = mouse.clone();
        let compiled = Rc::new(RefCell::new(false));
        let compiled2 = compiled.clone();
        let shader_for_compile = shader.clone();
        let gl_ctx: Rc<RefCell<Option<glow::Context>>> = Rc::new(RefCell::new(None));
        let gl_ctx2 = gl_ctx.clone();

        gl_area.connect_render(move |area, _ctx| {
            // Make sure GL context is current
            area.make_current();

            // Create glow context only once
            if gl_ctx2.borrow().is_none() {
                if let Err(e) = create_glow_context_into(&gl_ctx2) {
                    eprintln!("Failed to create GL context: {}", e);
                    return glib::Propagation::Stop;
                }
            }
            let gl_ref = gl_ctx2.borrow();
            let gl = match gl_ref.as_ref() {
                Some(g) => g,
                None => return glib::Propagation::Stop,
            };

            // Compile on first render
            if !*compiled2.borrow() {
                if let Err(e) = shader_for_compile.compile(gl) {
                    eprintln!("Shader compile error: {}", e);
                    return glib::Propagation::Stop;
                }
                *compiled2.borrow_mut() = true;
            }

            // Get viewport size
            let alloc = area.allocation();
            use glow::HasContext;
            unsafe { gl.viewport(0, 0, alloc.width(), alloc.height()); }

            let mut u = *uniforms_clone.borrow();
            if auto_animate {
                u.time = start_time.elapsed().as_secs_f32();
            }
            let (mx, my) = *mouse_clone.borrow();
            u.mouse_x = mx;
            u.mouse_y = my;
            u.resolution_x = alloc.width() as f32;
            u.resolution_y = alloc.height() as f32;
            *uniforms_clone.borrow_mut() = u;

            crate::shader::render_fullscreen(gl, &shader_for_compile, &u);

            glib::Propagation::Proceed
        });

        if auto_animate {
            // Drive redraws with a self-cancelling timer: it only holds a weak
            // reference to the GLArea and stops itself once the widget is
            // destroyed. A permanent timer with a strong reference would keep
            // the widget tree alive and pile up 60 FPS redraw sources every
            // time a ShaderView is created.
            let gl_area_weak = gl_area.downgrade();
            glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                let Some(area) = gl_area_weak.upgrade() else {
                    return glib::ControlFlow::Break;
                };
                area.queue_draw();
                glib::ControlFlow::Continue
            });
        }

        gl_area.upcast()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        available
    }
}

impl Widget for ShaderView {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }

    fn to_gtk(&self) -> gtk::Widget {
        let sv = ShaderView {
            id: self.id,
            shader: self.shader.clone(),
            uniforms: self.uniforms,
            auto_animate: self.auto_animate,
            position_mode: self.position_mode,
            position: self.position,
        };
        View::new(sv).to_gtk()
    }

    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> Padding { Padding::ZERO }
}

/// Create a glow::Context and store it in the RefCell.
pub fn create_glow_context_into(target: &Rc<RefCell<Option<glow::Context>>>) -> Result<(), String> {
    let get_proc = get_gl_proc_address().ok_or("Failed to get GL proc address")?;
    let gl = unsafe {
        glow::Context::from_loader_function_cstr(|name| {
            get_proc(name.as_ptr()) as *const std::os::raw::c_void
        })
    };
    *target.borrow_mut() = Some(gl);
    Ok(())
}

/// Get the GL function loader address — uses glXGetProcAddress from libGLX directly.
fn get_gl_proc_address() -> Option<unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_void> {
    use std::sync::OnceLock;
    static GET_PROC: OnceLock<unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_void> = OnceLock::new();
    Some(*GET_PROC.get_or_init(|| {
        unsafe {
            let lib = libloading::Library::new("libGLX.so.0")
                .or_else(|_| libloading::Library::new("libGLX.so"))
                .or_else(|_| libloading::Library::new("libGL.so.1"))
                .or_else(|_| libloading::Library::new("libGL.so"))
                .expect("Failed to load libGLX/libGL");
            let func: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_void> =
                lib.get(b"glXGetProcAddress\0")
                    .or_else(|_| lib.get(b"glXGetProcAddressARB\0"))
                    .expect("glXGetProcAddress symbol not found");
            let ptr = *func;
            std::mem::forget(lib);
            ptr
        }
    }))
}

// ═══════════════════════════════════════════════════════════════
// Convenience constructors
// ═══════════════════════════════════════════════════════════════

pub fn gradient_view(width: f32, height: f32) -> ShaderView {
    ShaderView::new(Shader::gradient()).size(width, height)
}

pub fn blur_view(width: f32, height: f32, sigma: f32) -> ShaderView {
    ShaderView::new(Shader::blur())
        .size(width, height)
        .uniforms(Uniforms { param1: sigma, ..Default::default() })
}

pub fn glass_view(width: f32, height: f32) -> ShaderView {
    ShaderView::new(Shader::glass())
        .size(width, height)
        .uniforms(Uniforms { param1: 0.5, param2: 0.7, ..Default::default() })
}

pub fn noise_view(width: f32, height: f32) -> ShaderView {
    ShaderView::new(Shader::noise()).size(width, height)
}

pub fn rainbow_view(width: f32, height: f32) -> ShaderView {
    ShaderView::new(Shader::rainbow()).size(width, height)
}

pub fn wave_view(width: f32, height: f32) -> ShaderView {
    ShaderView::new(Shader::wave()).size(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_view_creation() {
        let sv = ShaderView::new(Shader::gradient());
        assert!(sv.auto_animate);
    }

    #[test]
    fn shader_view_with_uniforms() {
        let sv = ShaderView::new(Shader::blur())
            .uniforms(Uniforms { param1: 5.0, ..Default::default() });
        assert_eq!(sv.uniforms.param1, 5.0);
    }
}
