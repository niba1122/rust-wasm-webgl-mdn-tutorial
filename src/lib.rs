mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, WebGlShader};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);

    #[wasm_bindgen(js_namespace = mat4, js_name = create)]
    fn mat4_create() -> js_sys::Float32Array;
}

static FRAGMENT_SHADER: &'static str = r#"
    void main(void) {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

static VERTEX_SHADER: &'static str = r#"
    attribute vec3 aVertexPosition;

    uniform mat4 uMVMatrix;
    uniform mat4 uPMatrix;

    void main(void) {
        gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);
    }
"#;


#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    let context = get_webgl_context_by_id("canvas");

    init_shaders(&context);

    init_buffers(&context);

    draw_scene(&context);

    Ok(())
}

fn get_webgl_context_by_id(id: &str) -> WebGlRenderingContext {
    let document = web_sys::window()
        .unwrap()
        .document()
        .unwrap();

    let canvas = document
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("webgl")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    context
}

fn get_shader(context: &WebGlRenderingContext, shader_type: u32, source: &str) -> WebGlShader {
    let shader = context.create_shader(shader_type).unwrap();

    context.shader_source(&shader, source);
    context.compile_shader(&shader);
    let compile_is_succeeded = context.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap();
    if !compile_is_succeeded {
        panic!("シェーダーのコンパイルでエラーが発生しました");
    }
    shader
}

fn init_shaders(context: &WebGlRenderingContext) {
    let fragment_shader = get_shader(&context, WebGlRenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER);
    let vertex_shader = get_shader(&context, WebGlRenderingContext::VERTEX_SHADER, VERTEX_SHADER);

    let shader_program = context.create_program().unwrap();
    context.attach_shader(&shader_program, &vertex_shader);
    context.attach_shader(&shader_program, &fragment_shader);
    context.link_program(&shader_program);

    let shader_is_created = context.get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap();

    if !shader_is_created {
        let info = context.get_program_info_log(&shader_program).unwrap();
        error(&format!("シェーダープログラムを初期化できません: {}", info));
    }

    context.use_program(Some(&shader_program));

    let vertex_position_attribute = context.get_attrib_location(&shader_program, "aVertexPosition");
    context.enable_vertex_attrib_array(vertex_position_attribute as u32);
}

// var horizAspect = 480.0/640.0;

fn init_buffers(context: &WebGlRenderingContext) {
    let square_vertices_buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&square_vertices_buffer));
    
    let vertices: &[f32] = &[
        1.0,  1.0,  0.0,
        -1.0, 1.0,  0.0,
        1.0,  -1.0, 0.0,
        -1.0, -1.0, 0.0
    ];
  
    context.buffer_data_with_array_buffer_view(WebGlRenderingContext::ARRAY_BUFFER, &js_sys::Float32Array::from(vertices), WebGlRenderingContext::STATIC_DRAW);
}

fn draw_scene(context: &WebGlRenderingContext) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.enable(WebGlRenderingContext::DEPTH_TEST);
    context.depth_func(WebGlRenderingContext::LEQUAL);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

    let canvas = context.canvas().unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    // let fieldOfView = 45.0 * std::f64::consts::PI / 180.0;
    // let aspect = canvas.client_width() / canvas.client_height();
    // let z_near = 0.1;
    // let z_far = 100.0;
    // let projectionMatrix = mat4_create();
}
