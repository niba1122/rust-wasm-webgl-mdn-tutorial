mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlBuffer, WebGlProgram, WebGlUniformLocation};

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

    #[wasm_bindgen]
    fn calc(field_of_view: f32, aspect: f32, z_near: f32, z_far: f32) -> js_sys::Object;
}

static FRAGMENT_SHADER: &'static str = r#"
    void main(void) {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

static VERTEX_SHADER: &'static str = r#"
    attribute vec4 aVertexPosition;

    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;

    void main(void) {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
    }
"#;


#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    let context = get_webgl_context_by_id("canvas");

    let shader_program = init_shaders(&context);

    let position_buffer = init_buffers(&context);
    let vertex_position = context.get_attrib_location(&shader_program, "aVertexPosition") as u32;
    let program_projection_matrix = context.get_uniform_location(&shader_program, "uProjectionMatrix").unwrap();
    let program_model_view_matrix = context.get_uniform_location(&shader_program, "uModelViewMatrix").unwrap();

    draw_scene(
        &context,
        &position_buffer,
        &shader_program,
        vertex_position,
        program_projection_matrix,
        program_model_view_matrix
    );

    Ok(())
}

fn get_webgl_context_by_id(id: &str) -> WebGl2RenderingContext {
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
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()
        .unwrap();

    context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    context
}

fn get_shader(context: &WebGl2RenderingContext, shader_type: u32, source: &str) -> WebGlShader {
    let shader = context.create_shader(shader_type).unwrap();

    context.shader_source(&shader, source);
    context.compile_shader(&shader);
    let compile_is_succeeded = context.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap();
    if !compile_is_succeeded {
        panic!("シェーダーのコンパイルでエラーが発生しました");
    }
    shader
}

fn init_shaders(context: &WebGl2RenderingContext) -> WebGlProgram {
    let fragment_shader = get_shader(&context, WebGl2RenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER);
    let vertex_shader = get_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, VERTEX_SHADER);

    let shader_program = context.create_program().unwrap();
    context.attach_shader(&shader_program, &vertex_shader);
    context.attach_shader(&shader_program, &fragment_shader);
    context.link_program(&shader_program);

    let shader_is_created = context.get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap();

    if !shader_is_created {
        let info = context.get_program_info_log(&shader_program).unwrap();
        error(&format!("シェーダープログラムを初期化できません: {}", info));
    }

    context.use_program(Some(&shader_program));

    let vertex_position_attribute = context.get_attrib_location(&shader_program, "aVertexPosition");
    context.enable_vertex_attrib_array(vertex_position_attribute as u32);

    shader_program
}

fn init_buffers(context: &WebGl2RenderingContext) -> WebGlBuffer {
    let position_buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
    
    let vertices: &[f32] = &[
        1.0,  1.0,
        -1.0, 1.0,
        1.0,  -1.0,
        -1.0, -1.0
    ];
  
    unsafe {
        context.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &js_sys::Float32Array::view(vertices), WebGl2RenderingContext::STATIC_DRAW);
    }

    position_buffer
}

fn draw_scene(
    context: &WebGl2RenderingContext,
    position_buffer: &WebGlBuffer,
    shader_program: &WebGlProgram,
    vertex_position: u32,
    program_projection_matrix: WebGlUniformLocation,
    program_model_view_matrix: WebGlUniformLocation
) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.enable(WebGl2RenderingContext::DEPTH_TEST);
    context.depth_func(WebGl2RenderingContext::LEQUAL);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

    let canvas = context.canvas().unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
    let aspect = canvas.client_width() as f32 / canvas.client_height() as f32;
    let z_near = 0.1;
    let z_far = 100.0;

    let result = calc(field_of_view, aspect, z_near, z_far);
    let raw_projection_matrix: js_sys::Float32Array = js_sys::Reflect::get(&result, &js_sys::JsString::from("projectionMatrix")).unwrap().into();
    let projection_matrix = js_sys::Float32Array::from(raw_projection_matrix).to_vec();

    let raw_model_view_matrix: js_sys::Float32Array = js_sys::Reflect::get(&result, &js_sys::JsString::from("modelViewMatrix")).unwrap().into();
    let model_view_matrix = js_sys::Float32Array::from(raw_model_view_matrix).to_vec();

    let num_components = 2;
    let rendering_type: u32 = WebGl2RenderingContext::FLOAT;
    let normalize = false;
    let stride = 0;
    let offset = 0;

    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
    context.vertex_attrib_pointer_with_i32(
        vertex_position,
        num_components,
        rendering_type,
        normalize,
        stride,
        offset
    );
    context.enable_vertex_attrib_array(vertex_position);
    context.use_program(Some(&shader_program));

    context.uniform_matrix4fv_with_f32_array(
        Some(&program_projection_matrix),
        false,
        &projection_matrix
    );

    context.uniform_matrix4fv_with_f32_array(
        Some(&program_model_view_matrix),
        false,
        &model_view_matrix
    );

    let offset = 0;
    let vertex_count = 4;
    context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, offset, vertex_count);

    log(&format!("vertex position: {:?}", vertex_position));
    log(&format!("projection matrix: \n{}", format_as_matrix(projection_matrix, 4, 4)));
    log(&format!("program projection matrix: {:?}", program_projection_matrix));
    log(&format!("model view matrix: \n{}", format_as_matrix(model_view_matrix, 4, 4)));
    log(&format!("program model view matrix: {:?}", program_model_view_matrix));
}

fn format_as_matrix<T: std::fmt::Display>(vec: Vec<T>, len_row: usize, len_column: usize) -> String {
    let len = vec.len();
    if len != len_column * len_row {
        panic!("vector couldn't be divided by len_row");
    }

    (0..len_row).into_iter().map(|i| {
        (0..len_column).into_iter().map(|j| {
            format!("{}", &vec[i*len_row+j])
        }).collect::<Vec<_>>().join(",")
    }).collect::<Vec<_>>().join("\n")
}

