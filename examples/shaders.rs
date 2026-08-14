//! Example 3: Shader Gallery
//!
//! Demonstrates all built-in shaders and custom shader effects.

use uikit::prelude::*;
use gtk::prelude::*;

struct ShaderCardView {
    title: String,
    shader: Shader,
    uniforms: Uniforms,
}

impl ViewContent for ShaderCardView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        vbox.set_hexpand(true);
        vbox.set_vexpand(false);

        let css = format!(
            "box {{ background-color: rgba(42, 42, 44, 0.9); border-radius: 12px; padding: 12px; border: 1px solid rgba(255, 255, 255, 0.08); }}"
        );
        uikit::apply_css(&vbox, &css);

        // Title
        let title_label = gtk::Label::new(Some(&self.title));
        title_label.set_halign(gtk::Align::Start);
        uikit::apply_css(&title_label, "label { font-family: 'SF Pro Display'; font-size: 14px; font-weight: bold; color: #ececec; }");
        vbox.append(&title_label);

        // Shader view
        let shader_view = ShaderView::new(self.shader.clone())
            .uniforms(self.uniforms)
            .size(frame.width - 24.0, 200.0);
        let shader_widget = View::new(shader_view).with_frame(0.0, 0.0, frame.width - 24.0, 200.0);
        vbox.append(&shader_widget.to_gtk());

        if frame.width > 0.0 {
            vbox.set_width_request(frame.width as i32);
        }

        vbox.upcast()
    }
}

struct ShaderApp;

impl AppDelegate for ShaderApp {
    fn view(&self) -> Box<dyn Widget> {
        let mut root = View::empty();
        root.set_frame(0.0, 0.0, 600.0, 900.0);

        // Title
        let title = Label::new("Shader Gallery").font_size(28.0).bold().color(Color::WHITE);
        root.add_subview(View::new(title).with_frame(24.0, 0.0, 300.0, 36.0));

        // Gradient
        let gradient_card = ShaderCardView {
            title: "Animated Gradient".into(),
            shader: Shader::gradient(),
            uniforms: Uniforms { param1: 1.0, ..Default::default() },
        };
        root.add_subview(View::new(gradient_card).with_frame(24.0, 0.0, 552.0, 260.0));

        // Rainbow
        let rainbow_card = ShaderCardView {
            title: "Rainbow Flow".into(),
            shader: Shader::rainbow(),
            uniforms: Uniforms { param1: 0.5, ..Default::default() },
        };
        root.add_subview(View::new(rainbow_card).with_frame(24.0, 0.0, 552.0, 260.0));

        // Wave
        let wave_card = ShaderCardView {
            title: "Wave Distortion".into(),
            shader: Shader::wave(),
            uniforms: Uniforms { param1: 1.0, param2: 1.0, param3: 1.0, ..Default::default() },
        };
        root.add_subview(View::new(wave_card).with_frame(24.0, 0.0, 552.0, 260.0));

        // Glass
        let glass_card = ShaderCardView {
            title: "LiquidGlass".into(),
            shader: Shader::glass(),
            uniforms: Uniforms { param1: 0.5, param2: 0.7, ..Default::default() },
        };
        root.add_subview(View::new(glass_card).with_frame(24.0, 0.0, 552.0, 260.0));

        // Noise
        let noise_card = ShaderCardView {
            title: "Animated Noise".into(),
            shader: Shader::noise(),
            uniforms: Uniforms { param1: 0.5, ..Default::default() },
        };
        root.add_subview(View::new(noise_card).with_frame(24.0, 0.0, 552.0, 260.0));

        // Blur
        let blur_card = ShaderCardView {
            title: "Gaussian Blur".into(),
            shader: Shader::blur(),
            uniforms: Uniforms { param1: 3.0, ..Default::default() },
        };
        root.add_subview(View::new(blur_card).with_frame(24.0, 0.0, 552.0, 260.0));

        // Custom shader
        let custom = Shader::from_source(
            VERTEX_FULLSCREEN,
            r#"#version 330
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
}
"#,
        ).unwrap();

        let custom_card = ShaderCardView {
            title: "Custom: Radial Pulse".into(),
            shader: custom,
            uniforms: Uniforms::new(),
        };
        root.add_subview(View::new(custom_card).with_frame(24.0, 0.0, 552.0, 260.0));

        struct W(View);
        impl Widget for W {
            fn id(&self) -> uikit::widget::WidgetId { 0 }
            fn to_gtk(&self) -> gtk::Widget { self.0.to_gtk_scrollable() }
        }
        Box::new(W(root))
    }
}

fn main() {
    let mut app = App::new("Shader Gallery", 600, 900);
    app.set_color_scheme(ColorScheme::Dark);
    app.set_delegate(ShaderApp);
    app.run();
}
