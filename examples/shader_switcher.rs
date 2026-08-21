//! Example 4: Shader Switcher
//!
//! A button cycles through all built-in shaders with a live preview.

use uikit::prelude::*;
use uikit::shader::{Shader, Uniforms, VERTEX_FULLSCREEN};
use uikit::shader_view::create_glow_context_into;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn shader_name(idx: usize) -> &'static str {
    match idx {
        0 => "Gradient",
        1 => "Rainbow",
        2 => "Wave",
        3 => "Glass",
        4 => "Noise",
        5 => "Blur",
        _ => "Custom: Radial Pulse",
    }
}

fn make_shader(idx: usize) -> Shader {
    match idx {
        0 => Shader::gradient(),
        1 => Shader::rainbow(),
        2 => Shader::wave(),
        3 => Shader::glass(),
        4 => Shader::noise(),
        5 => Shader::blur(),
        _ => Shader::from_source(
            VERTEX_FULLSCREEN,
            r#"#version 300 es
            precision mediump float;
            in vec2 v_uv;
            out vec4 fragColor;
            uniform float u_time;
            uniform vec2 u_resolution;
            uniform vec4 u_params;

            void main() {
                vec2 uv = v_uv;
                vec2 center = vec2(0.5);
                float d = length(uv - center);
                float angle = atan(uv.y - 0.5, uv.x - 0.5);
                float r = sin(d * 20.0 - u_time * 2.0) * 0.5 + 0.5;
                float g = sin(angle * 3.0 + u_time) * 0.5 + 0.5;
                float b = cos(d * 15.0 + u_time * 1.5) * 0.5 + 0.5;
                vec3 color = vec3(r * 0.8, g * 0.4 + 0.3, b * 0.6 + 0.2);
                float a = smoothstep(0.5, 0.3, d);
                fragColor = vec4(color * a, a);
            }"#
        ).unwrap(),
    }
}

struct ShaderSwitcherApp;

impl AppDelegate for ShaderSwitcherApp {
    fn view(&self) -> Box<dyn Widget> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.set_hexpand(true);
        vbox.set_vexpand(true);

        // Shared state
        let shader_idx = Rc::new(RefCell::new(0usize));
        let current_shader = Rc::new(RefCell::new(Some(make_shader(0))));
        let compiled = Rc::new(RefCell::new(false));
        let gl_ctx: Rc<RefCell<Option<glow::Context>>> = Rc::new(RefCell::new(None));

        // Title bar
        let title_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        title_bar.set_hexpand(true);
        title_bar.set_height_request(48);
        uikit::apply_css(&title_bar, "box { background-color: rgba(28, 28, 30, 0.95); padding: 0 16px; }");

        let name_label = gtk::Label::new(Some(shader_name(0)));
        name_label.set_hexpand(true);
        name_label.set_halign(gtk::Align::Start);
        uikit::apply_css(&name_label, "label { font-family: 'SF Pro Display'; font-size: 16px; font-weight: bold; color: #ececec; }");
        title_bar.append(&name_label);

        let counter_label = gtk::Label::new(Some("1/7"));
        uikit::apply_css(&counter_label, "label { font-family: 'SF Pro Display'; font-size: 13px; color: #8e8e93; }");
        title_bar.append(&counter_label);

        vbox.append(&title_bar);

        // GLArea
        let gl_area = gtk::GLArea::new();
        gl_area.set_hexpand(true);
        gl_area.set_vexpand(true);
        gl_area.set_required_version(3, 0);
        gl_area.set_auto_render(true);

        let current_r = current_shader.clone();
        let compiled_r = compiled.clone();
        let gl_ctx_r = gl_ctx.clone();
        let start_time = std::time::Instant::now();

        gl_area.connect_render(move |area, _ctx| {
            area.make_current();

            if gl_ctx_r.borrow().is_none() {
                if let Err(e) = create_glow_context_into(&gl_ctx_r) {
                    eprintln!("Failed to create GL context: {}", e);
                    return glib::Propagation::Stop;
                }
            }
            let gl_ref = gl_ctx_r.borrow();
            let gl = match gl_ref.as_ref() {
                Some(g) => g,
                None => return glib::Propagation::Stop,
            };

            // Compile if needed
            if !*compiled_r.borrow() {
                if let Some(ref shader) = *current_r.borrow() {
                    if let Err(e) = shader.compile(gl) {
                        eprintln!("Shader compile error: {}", e);
                        return glib::Propagation::Stop;
                    }
                    *compiled_r.borrow_mut() = true;
                }
            }

            let alloc = area.allocation();
            use glow::HasContext;
            unsafe { gl.viewport(0, 0, alloc.width(), alloc.height()); }

            let uniforms = Uniforms {
                time: start_time.elapsed().as_secs_f32(),
                resolution_x: alloc.width() as f32,
                resolution_y: alloc.height() as f32,
                param1: 1.0,
                param2: 0.7,
                param3: 1.0,
                param4: 0.0,
                ..Default::default()
            };

            if let Some(ref shader) = *current_r.borrow() {
                uikit::shader::render_fullscreen(gl, shader, &uniforms);
            }

            glib::Propagation::Proceed
        });

        {
            let gl_area_weak = gl_area.downgrade();
            glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                let Some(area) = gl_area_weak.upgrade() else {
                    return glib::ControlFlow::Break;
                };
                area.queue_draw();
                glib::ControlFlow::Continue
            });
        }

        vbox.append(&gl_area);

        // Button bar
        let btn_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        btn_bar.set_hexpand(true);
        btn_bar.set_height_request(60);
        btn_bar.set_halign(gtk::Align::Center);
        btn_bar.set_valign(gtk::Align::End);
        btn_bar.set_margin_bottom(16);

        let next_btn = gtk::Button::with_label("Next Shader");
        next_btn.set_size_request(160, 44);
        uikit::apply_css(&next_btn,
            "button {
                background: #FF6B2B;
                color: white;
                border-radius: 10px;
                font-family: 'SF Pro Display';
                font-size: 14px;
                font-weight: bold;
                padding: 8px 24px;
            }
            button:hover { filter: brightness(1.1); }
            button:active { filter: brightness(0.9); }"
        );

        {
            let name_ref = name_label.clone();
            let counter_ref = counter_label.clone();
            let idx_ref = shader_idx.clone();
            let current_ref = current_shader.clone();
            let compiled_ref = compiled.clone();

            next_btn.connect_clicked(move |_| {
                let mut idx = idx_ref.borrow_mut();
                *idx = (*idx + 1) % 7;
                let new_idx = *idx;
                drop(idx);

                name_ref.set_text(shader_name(new_idx));
                counter_ref.set_text(&format!("{}/7", new_idx + 1));

                // Replace shader and force recompile
                *current_ref.borrow_mut() = Some(make_shader(new_idx));
                *compiled_ref.borrow_mut() = false;
            });
        }

        btn_bar.append(&next_btn);
        vbox.append(&btn_bar);

        struct W(gtk::Box);
        impl Widget for W {
            fn id(&self) -> uikit::widget::WidgetId { 0 }
            fn to_gtk(&self) -> gtk::Widget { self.0.clone().upcast() }
        }
        Box::new(W(vbox))
    }
}

fn main() {
    let mut app = App::new("Shader Switcher", 700, 500);
    app.set_color_scheme(ColorScheme::Dark);
    app.set_delegate(ShaderSwitcherApp);
    app.run();
}
