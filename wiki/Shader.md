# Shader

GPU-accelerated shader rendering using GTK4's GLArea and the `glow` OpenGL bindings. Provides GLSL shader compilation, built-in effects, and a `ShaderView` widget for embedding shaders in UIKit apps.

## Shader

### `Shader::from_source(vertex, fragment)`

Create a shader from GLSL source code. Returns `Result<Shader, ShaderError>`.

```rust
let shader = Shader::from_source(
    VERTEX_FULLSCREEN,
    r#"#version 300 es
    precision mediump float;
    in vec2 v_uv;
    out vec4 fragColor;
    uniform float u_time;
    void main() {
        fragColor = vec4(vec3(v_uv.x, v_uv.y, sin(u_time)), 1.0);
    }"#
)?;
```

### Built-in Shaders

| Shader | Description | Key Uniforms |
|---|---|---|
| `Shader::gradient()` | Animated gradient | `u_params.x` = angle offset |
| `Shader::blur()` | Gaussian blur | `u_params.x` = sigma (0-10) |
| `Shader::glass()` | LiquidGlass / frosted | `u_params.x` = blur, `u_params.y` = opacity |
| `Shader::noise()` | Animated noise | `u_params.x` = speed |
| `Shader::rainbow()` | Rainbow flow | `u_params.x` = speed |
| `Shader::wave()` | Wave distortion | `u_params.xyz` = amplitude, frequency, speed |

### Shader Methods

| Method | Description |
|---|---|
| `compile(&gl)` | Compile with GL context |
| `use_program(&gl)` | Bind shader |
| `set_uniform_f32(gl, name, value)` | Set float uniform |
| `set_uniform_vec2(gl, name, x, y)` | Set vec2 uniform |
| `set_uniform_vec3(gl, name, x, y, z)` | Set vec3 uniform |
| `set_uniform_vec4(gl, name, x, y, z, w)` | Set vec4 uniform |
| `set_uniform_i32(gl, name, value)` | Set int uniform |
| `destroy(&gl)` | Delete shader program |
| `is_compiled()` | Check if compiled |

## Uniforms

Standard uniforms passed to every shader:

```rust
pub struct Uniforms {
    pub time: f32,          // seconds since start
    pub resolution_x: f32,  // viewport width
    pub resolution_y: f32,  // viewport height
    pub mouse_x: f32,       // mouse X position
    pub mouse_y: f32,       // mouse Y position
    pub param1: f32,        // custom parameter
    pub param2: f32,        // custom parameter
    pub param3: f32,        // custom parameter
    pub param4: f32,        // custom parameter
}
```

All uniforms are declared as `uniform` in GLSL:

```glsl
uniform float u_time;
uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform vec4 u_params;
```

## ShaderView

Widget that renders a shader in a GLArea. Auto-animates at 60 FPS and tracks mouse position.

```rust
let shader_view = ShaderView::new(Shader::gradient())
    .size(400.0, 300.0)
    .uniforms(Uniforms { param1: 1.0, ..Default::default() })
    .auto_animate(true);

let view = View::new(shader_view).with_frame(0.0, 0.0, 400.0, 300.0);
```

### ShaderView Methods

| Method | Description |
|---|---|
| `new(shader)` | Create with a shader |
| `size(w, h)` | Set dimensions |
| `uniforms(Uniforms)` | Set initial uniforms |
| `auto_animate(bool)` | Enable/disable auto-animation |
| `to_view(x, y, w, h)` | Convert to View |

### Convenience Constructors

```rust
gradient_view(width, height)         // Animated gradient
blur_view(width, height, sigma)      // Gaussian blur
glass_view(width, height)            // LiquidGlass effect
noise_view(width, height)            // Animated noise
rainbow_view(width, height)          // Rainbow flow
wave_view(width, height)             // Wave distortion
```

## Vertex Shader

All built-in shaders use a fullscreen quad vertex shader:

```rust
pub const VERTEX_FULLSCREEN: &str = r#"#version 300 es
precision mediump float;
in vec2 a_position;
out vec2 v_uv;
void main() {
    v_uv = a_position * 0.5 + 0.5;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
```

## Custom Shader Example

```rust
let custom = Shader::from_source(
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
        float d = length(uv - 0.5);
        float pulse = sin(d * 20.0 - u_time * 2.0) * 0.5 + 0.5;
        vec3 color = vec3(pulse * 0.8, pulse * 0.4, pulse * 0.6);
        float a = smoothstep(0.5, 0.3, d);
        fragColor = vec4(color * a, a);
    }"#
).unwrap();

let view = ShaderView::new(custom)
    .size(400.0, 300.0)
    .to_view(0.0, 0.0, 400.0, 300.0);
```

## render_fullscreen Helper

Low-level function to render a fullscreen quad with a shader:

```rust
pub fn render_fullscreen(gl: &glow::Context, shader: &Shader, uniforms: &Uniforms)
```

Sets up VAO/VBO, binds shader, applies uniforms, and draws 2 triangles covering the viewport.

## ShaderError

```rust
pub enum ShaderError {
    CreateProgram(String),
    CreateShader(String),
    CompileError(String),
    LinkError(String),
}
```

## Technical Details

- Uses `glow` (OpenGL bindings) for shader compilation
- Uses `libloading` + `glXGetProcAddress` from libGLX for GL function loading
- GLArea creates an OpenGL ES 3.0 context (GTK4 default); all shaders use `#version 300 es`
- Auto-animation via a self-cancelling `glib::timeout_add_local` timer at 60 FPS
  (weak reference to the `GLArea`; stops when the widget is destroyed)
- Mouse tracking via `gtk::EventControllerMotion`

## Cross References

- [View.md](View.md) -- View base class
- [Widgets.md](Widgets.md) -- other widgets
- [MAIN.md](MAIN.md) -- overview
