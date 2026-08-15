//! Shader loading, compilation, and management for OpenGL rendering.
//!
//! Provides a safe abstraction over GLSL shader compilation using `glow`.
//! Includes built-in shaders for common effects (blur, gradient, glass, noise).

use glow::HasContext;
use std::sync::{Arc, Mutex};

// Type aliases for glow's associated types
type Gl = glow::Context;
type GlProgram = <glow::Context as HasContext>::Program;
type GlShader = <glow::Context as HasContext>::Shader;
type GlUniformLocation = <glow::Context as HasContext>::UniformLocation;
type GlVertexArray = <glow::Context as HasContext>::VertexArray;
type GlBuffer = <glow::Context as HasContext>::Buffer;

// ═══════════════════════════════════════════════════════════════
// Shader
// ═══════════════════════════════════════════════════════════════

/// A compiled OpenGL shader program.
#[derive(Clone)]
pub struct Shader {
    inner: Arc<Mutex<ShaderInner>>,
}

struct ShaderInner {
    program: Option<GlProgram>,
    vertex_src: String,
    fragment_src: String,
}

impl Shader {
    /// Create a shader from GLSL source code.
    pub fn from_source(vertex_src: &str, fragment_src: &str) -> Result<Self, ShaderError> {
        Ok(Self {
            inner: Arc::new(Mutex::new(ShaderInner {
                program: None,
                vertex_src: vertex_src.to_string(),
                fragment_src: fragment_src.to_string(),
            })),
        })
    }

    /// Compile the shader. Must be called on the GL thread.
    pub fn compile(&self, gl: &Gl) -> Result<(), ShaderError> {
        let mut inner = self.inner.lock().unwrap();

        let program = unsafe { gl.create_program().map_err(ShaderError::CreateProgram)? };

        let vert: GlShader = unsafe { gl.create_shader(glow::VERTEX_SHADER).map_err(ShaderError::CreateShader)? };
        unsafe { gl.shader_source(vert, &inner.vertex_src); }
        unsafe { gl.compile_shader(vert); }
        if !unsafe { gl.get_shader_compile_status(vert) } {
            let log = unsafe { gl.get_shader_info_log(vert) };
            unsafe { gl.delete_shader(vert); }
            unsafe { gl.delete_program(program); }
            return Err(ShaderError::CompileError(log));
        }

        let frag: GlShader = unsafe { gl.create_shader(glow::FRAGMENT_SHADER).map_err(ShaderError::CreateShader)? };
        unsafe { gl.shader_source(frag, &inner.fragment_src); }
        unsafe { gl.compile_shader(frag); }
        if !unsafe { gl.get_shader_compile_status(frag) } {
            let log = unsafe { gl.get_shader_info_log(frag) };
            unsafe { gl.delete_shader(vert); }
            unsafe { gl.delete_shader(frag); }
            unsafe { gl.delete_program(program); }
            return Err(ShaderError::CompileError(log));
        }

        unsafe {
            gl.attach_shader(program, vert);
            gl.attach_shader(program, frag);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                gl.delete_shader(vert);
                gl.delete_shader(frag);
                gl.delete_program(program);
                return Err(ShaderError::LinkError(log));
            }
            gl.detach_shader(program, vert);
            gl.detach_shader(program, frag);
            gl.delete_shader(vert);
            gl.delete_shader(frag);
        }

        inner.program = Some(program);
        Ok(())
    }

    /// Use this shader program.
    pub fn use_program(&self, gl: &Gl) {
        let inner = self.inner.lock().unwrap();
        if let Some(program) = inner.program {
            unsafe { gl.use_program(Some(program)); }
        }
    }

    /// Get the uniform location.
    pub fn uniform_location(&self, gl: &Gl, name: &str) -> Option<GlUniformLocation> {
        let inner = self.inner.lock().unwrap();
        inner.program.and_then(|p| unsafe { gl.get_uniform_location(p, name) })
    }

    /// Set a float uniform.
    pub fn set_uniform_f32(&self, gl: &Gl, name: &str, value: f32) {
        if let Some(loc) = self.uniform_location(gl, name) {
            unsafe { gl.uniform_1_f32(Some(&loc), value); }
        }
    }

    /// Set a vec2 uniform.
    pub fn set_uniform_vec2(&self, gl: &Gl, name: &str, x: f32, y: f32) {
        if let Some(loc) = self.uniform_location(gl, name) {
            unsafe { gl.uniform_2_f32(Some(&loc), x, y); }
        }
    }

    /// Set a vec3 uniform.
    pub fn set_uniform_vec3(&self, gl: &Gl, name: &str, x: f32, y: f32, z: f32) {
        if let Some(loc) = self.uniform_location(gl, name) {
            unsafe { gl.uniform_3_f32(Some(&loc), x, y, z); }
        }
    }

    /// Set a vec4 uniform.
    pub fn set_uniform_vec4(&self, gl: &Gl, name: &str, x: f32, y: f32, z: f32, w: f32) {
        if let Some(loc) = self.uniform_location(gl, name) {
            unsafe { gl.uniform_4_f32(Some(&loc), x, y, z, w); }
        }
    }

    /// Set an int uniform.
    pub fn set_uniform_i32(&self, gl: &Gl, name: &str, value: i32) {
        if let Some(loc) = self.uniform_location(gl, name) {
            unsafe { gl.uniform_1_i32(Some(&loc), value); }
        }
    }

    /// Delete the shader program.
    pub fn destroy(&self, gl: &Gl) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(program) = inner.program.take() {
            unsafe { gl.delete_program(program); }
        }
    }

    /// Check if the shader has been compiled.
    pub fn is_compiled(&self) -> bool {
        self.inner.lock().unwrap().program.is_some()
    }

    pub fn vertex_source(&self) -> String {
        self.inner.lock().unwrap().vertex_src.clone()
    }

    pub fn fragment_source(&self) -> String {
        self.inner.lock().unwrap().fragment_src.clone()
    }
}

// ═══════════════════════════════════════════════════════════════
// ShaderError
// ═══════════════════════════════════════════════════════════════

#[derive(Debug)]
pub enum ShaderError {
    CreateProgram(String),
    CreateShader(String),
    CompileError(String),
    LinkError(String),
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateProgram(e) => write!(f, "Failed to create program: {}", e),
            Self::CreateShader(e) => write!(f, "Failed to create shader: {}", e),
            Self::CompileError(e) => write!(f, "Shader compile error: {}", e),
            Self::LinkError(e) => write!(f, "Program link error: {}", e),
        }
    }
}

impl std::error::Error for ShaderError {}

// ═══════════════════════════════════════════════════════════════
// Uniforms
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, Default)]
pub struct Uniforms {
    pub time: f32,
    pub resolution_x: f32,
    pub resolution_y: f32,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
    pub param4: f32,
}

impl Uniforms {
    pub fn new() -> Self { Self::default() }

    pub fn apply(&self, shader: &Shader, gl: &Gl) {
        shader.set_uniform_f32(gl, "u_time", self.time);
        shader.set_uniform_vec2(gl, "u_resolution", self.resolution_x, self.resolution_y);
        shader.set_uniform_vec2(gl, "u_mouse", self.mouse_x, self.mouse_y);
        shader.set_uniform_vec4(gl, "u_params", self.param1, self.param2, self.param3, self.param4);
    }
}

// ═══════════════════════════════════════════════════════════════
// Vertex shader
// ═══════════════════════════════════════════════════════════════

pub const VERTEX_FULLSCREEN: &str = r#"#version 300 es
precision mediump float;
in vec2 a_position;
out vec2 v_uv;
void main() {
    v_uv = a_position * 0.5 + 0.5;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

// ═══════════════════════════════════════════════════════════════
// Built-in shaders
// ═══════════════════════════════════════════════════════════════

impl Shader {
    pub fn gradient() -> Self { Self::from_source(VERTEX_FULLSCREEN, GRADIENT_FRAGMENT).unwrap() }
    pub fn blur() -> Self { Self::from_source(VERTEX_FULLSCREEN, BLUR_FRAGMENT).unwrap() }
    pub fn glass() -> Self { Self::from_source(VERTEX_FULLSCREEN, GLASS_FRAGMENT).unwrap() }
    pub fn noise() -> Self { Self::from_source(VERTEX_FULLSCREEN, NOISE_FRAGMENT).unwrap() }
    pub fn rainbow() -> Self { Self::from_source(VERTEX_FULLSCREEN, RAINBOW_FRAGMENT).unwrap() }
    pub fn wave() -> Self { Self::from_source(VERTEX_FULLSCREEN, WAVE_FRAGMENT).unwrap() }
}

// ═══════════════════════════════════════════════════════════════
// Built-in fragment shaders
// ═══════════════════════════════════════════════════════════════

const GRADIENT_FRAGMENT: &str = r#"#version 300 es
precision mediump float;
in vec2 v_uv;
out vec4 fragColor;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_params;

void main() {
    vec2 uv = v_uv;
    float angle = u_time * 0.3 + u_params.x;
    vec2 dir = vec2(cos(angle), sin(angle));
    float d = dot(uv, dir) * 0.5 + 0.5;
    vec3 c1 = vec3(1.0, 0.42, 0.17);
    vec3 c2 = vec3(0.05, 0.52, 0.94);
    vec3 c3 = vec3(0.73, 0.33, 0.89);
    vec3 color = d < 0.5 ? mix(c1, c2, d * 2.0) : mix(c2, c3, (d - 0.5) * 2.0);
    fragColor = vec4(color, 1.0);
}
"#;

const BLUR_FRAGMENT: &str = r#"#version 300 es
precision mediump float;
in vec2 v_uv;
out vec4 fragColor;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_params;

float gaussian(float x, float s) { return exp(-0.5*x*x/(s*s)); }

void main() {
    vec2 uv = v_uv;
    float sigma = u_params.x * 10.0 + 1.0;
    vec2 texel = 1.0 / u_resolution;
    vec3 color = vec3(0.0);
    float total = 0.0;
    for (float x = -sigma; x <= sigma; x += 1.0) {
        for (float y = -sigma; y <= sigma; y += 1.0) {
            float w = gaussian(length(vec2(x,y)), sigma);
            vec2 off = vec2(x,y) * texel;
            float p = sin((uv.x+off.x)*20.0+u_time) * cos((uv.y+off.y)*20.0+u_time*0.7)*0.5+0.5;
            color += vec3(p) * w;
            total += w;
        }
    }
    fragColor = vec4(color/total, 1.0);
}
"#;

const GLASS_FRAGMENT: &str = r#"#version 300 es
precision mediump float;
in vec2 v_uv;
out vec4 fragColor;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_params;

float noise(vec2 p) { return fract(sin(dot(p,vec2(12.9898,78.233)))*43758.5453); }

void main() {
    vec2 uv = v_uv;
    float sigma = u_params.x;
    float alpha = u_params.y;
    float n = noise(uv*50.0+u_time*0.5)*0.15;
    vec3 tint = vec3(0.85, 0.88, 0.92);
    float ba = sigma*0.02;
    vec2 off = vec2(sin(uv.y*30.0+u_time)*ba, cos(uv.x*30.0+u_time*0.7)*ba);
    float bd = sin((uv.x+off.x)*10.0)*cos((uv.y+off.y)*10.0)*0.5+0.5;
    vec3 color = mix(tint*bd, tint, n);
    fragColor = vec4(color, alpha+n*0.1);
}
"#;

const NOISE_FRAGMENT: &str = r#"#version 300 es
precision mediump float;
in vec2 v_uv;
out vec4 fragColor;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_params;

float hash(vec2 p) { return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453123); }
float noise(vec2 p) {
    vec2 i=floor(p), f=fract(p);
    f=f*f*(3.0-2.0*f);
    return mix(mix(hash(i),hash(i+vec2(1,0)),f.x),mix(hash(i+vec2(0,1)),hash(i+vec2(1,1)),f.x),f.y);
}
float fbm(vec2 p) {
    float v=0.0, a=0.5;
    for(int i=0;i<5;i++){v+=a*noise(p);p*=2.0;a*=0.5;}
    return v;
}

void main() {
    vec2 uv = v_uv;
    float speed = u_params.x*0.5+0.1;
    float n = fbm(uv*4.0+u_time*speed);
    float n2 = fbm(uv*8.0-u_time*speed*0.7);
    vec3 c1=vec3(0.11), c2=vec3(0.18,0.18,0.20), c3=vec3(0.25,0.25,0.28);
    vec3 color = mix(mix(c1,c2,n), c3, n2*0.5);
    fragColor = vec4(color, 1.0);
}
"#;

const RAINBOW_FRAGMENT: &str = r#"#version 300 es
precision mediump float;
in vec2 v_uv;
out vec4 fragColor;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_params;

vec3 hsv2rgb(vec3 c) {
    vec4 K=vec4(1.0,2.0/3.0,1.0/3.0,3.0);
    vec3 p=abs(fract(c.xxx+K.xyz)*6.0-K.www);
    return c.z*mix(K.xxx,clamp(p-K.xxx,0.0,1.0),c.y);
}

void main() {
    vec2 uv = v_uv;
    float speed = u_params.x*0.3+0.2;
    float hue = fract(uv.x+uv.y*0.3+u_time*speed);
    vec3 color = hsv2rgb(vec3(hue, 0.8+sin(u_time*0.5)*0.2, 0.9));
    fragColor = vec4(color, 1.0);
}
"#;

const WAVE_FRAGMENT: &str = r#"#version 300 es
precision mediump float;
in vec2 v_uv;
out vec4 fragColor;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_params;

void main() {
    vec2 uv = v_uv;
    float amp = u_params.x*0.05+0.02;
    float freq = u_params.y*10.0+5.0;
    float spd = u_params.z*2.0+1.0;
    float w = sin(uv.x*freq+u_time*spd)*amp;
    float w2 = cos(uv.y*freq*0.7+u_time*spd*0.8)*amp*0.5;
    vec2 d = uv + vec2(0.0, w+w2);
    vec3 c1=vec3(0.05,0.52,0.94), c2=vec3(0.73,0.33,0.89), c3=vec3(1.0,0.42,0.17);
    float t = d.x+d.y*0.5;
    vec3 color = t<0.5 ? mix(c1,c2,t*2.0) : mix(c2,c3,(t-0.5)*2.0);
    fragColor = vec4(color, 1.0);
}
"#;

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

/// Render a fullscreen quad using the given shader.
///
/// The VAO and VBO are created once and reused across frames.
pub fn render_fullscreen(gl: &Gl, shader: &Shader, uniforms: &Uniforms) {
    shader.use_program(gl);
    uniforms.apply(shader, gl);

    #[rustfmt::skip]
    let vertices: [f32; 12] = [
        -1.0, -1.0,  1.0, -1.0, -1.0,  1.0,
        -1.0,  1.0,  1.0, -1.0,  1.0,  1.0,
    ];

    thread_local! {
        static QUAD_STATE: std::cell::Cell<Option<(GlVertexArray, GlBuffer)>> = const { std::cell::Cell::new(None) };
    }

    QUAD_STATE.with(|cell| {
        if let Some((vao, vbo)) = cell.get() {
            unsafe {
                gl.bind_vertex_array(Some(vao));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
                gl.bind_vertex_array(None);
            }
        } else {
            unsafe {
                let new_vao = gl.create_vertex_array().unwrap();
                let new_vbo = gl.create_buffer().unwrap();

                gl.bind_vertex_array(Some(new_vao));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(new_vbo));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * 4),
                    glow::STATIC_DRAW,
                );

                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
                gl.bind_vertex_array(None);

                cell.set(Some((new_vao, new_vbo)));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_from_source() {
        let s = Shader::from_source(VERTEX_FULLSCREEN, GRADIENT_FRAGMENT).unwrap();
        assert!(!s.is_compiled());
        assert_eq!(s.vertex_source(), VERTEX_FULLSCREEN);
    }

    #[test]
    fn uniforms_default() {
        let u = Uniforms::new();
        assert_eq!(u.time, 0.0);
    }

    #[test]
    fn built_in_shaders_exist() {
        let _ = Shader::gradient();
        let _ = Shader::blur();
        let _ = Shader::glass();
        let _ = Shader::noise();
        let _ = Shader::rainbow();
        let _ = Shader::wave();
    }
}
