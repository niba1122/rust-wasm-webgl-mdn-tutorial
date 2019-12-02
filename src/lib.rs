mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, WebGlShader, WebGlBuffer, WebGlProgram, WebGlUniformLocation};
use std::rc::{Rc};
use std::cell::{RefCell};

extern crate nalgebra_glm as glm;

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
}

static FRAGMENT_SHADER: &'static str = r#"
    varying highp vec3 vLighting;

    void main(void) {
        gl_FragColor = vec4(vec3(1, 1, 1) * vLighting, 1);
    }
"#;

static VERTEX_SHADER: &'static str = r#"
    attribute vec4 aVertexPosition;
    attribute vec3 aVertexNormal;

    uniform mat4 uNormalMatrix;
    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;

    varying highp vec3 vLighting;

    void main(void) {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
        // Apply lighting effect
        highp vec3 ambientLight = vec3(0.3, 0.3, 0.3);
        highp vec3 directionalLightColor = vec3(1, 1, 1);
        highp vec3 directionalVector = normalize(vec3(0.85, 0.8, 0.75));
        highp vec4 transformedNormal = uNormalMatrix * vec4(aVertexNormal, 1.0);
        highp float directional = max(dot(transformedNormal.xyz, directionalVector), 0.0);
        vLighting = ambientLight + (directionalLightColor * directional);
    }
"#;

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    let context = get_webgl_context_by_id("canvas");

    let shader_program = init_shaders(&context);

    let (
        position_buffer,
        cube_vertices_index_buffer,
        cube_vertices_normal_buffer
    ) = init_buffers(&context);
    let vertex_position = context.get_attrib_location(&shader_program, "aVertexPosition") as u32;
    let vertex_normal = context.get_attrib_location(&shader_program, "aVertexNormal") as u32;
    let program_projection_matrix = context.get_uniform_location(&shader_program, "uProjectionMatrix").unwrap();
    let program_model_view_matrix = context.get_uniform_location(&shader_program, "uModelViewMatrix").unwrap();
    let program_normal_matrix = context.get_uniform_location(&shader_program, "uNormalMatrix").unwrap();
    let start_time = get_current_time();

    {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            draw_scene(
                &context,
                &shader_program,
                vertex_position,
                vertex_normal,
                &program_projection_matrix,
                &program_model_view_matrix,
                &program_normal_matrix,
                &position_buffer,
                &cube_vertices_index_buffer,
                &cube_vertices_normal_buffer,
                start_time,
                get_current_time()
            );

            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }

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

fn init_shaders(context: &WebGlRenderingContext) -> WebGlProgram {
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

    shader_program
}

fn init_buffers(context: &WebGlRenderingContext) -> (WebGlBuffer, WebGlBuffer, WebGlBuffer) {
    let vertices = [
        // 前面
        -1.0, -1.0,  1.0,
        1.0, -1.0,  1.0,
        1.0,  1.0,  1.0,
        -1.0,  1.0,  1.0,

        // 背面
        -1.0, -1.0, -1.0,
        -1.0,  1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0, -1.0, -1.0,

        // 上面
        -1.0,  1.0, -1.0,
        -1.0,  1.0,  1.0,
        1.0,  1.0,  1.0,
        1.0,  1.0, -1.0,

        // 底面
        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        1.0, -1.0,  1.0,
        -1.0, -1.0,  1.0,

        // 右側面
        1.0, -1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0,  1.0,  1.0,
        1.0, -1.0,  1.0,

        // 左側面
        -1.0, -1.0, -1.0,
        -1.0, -1.0,  1.0,
        -1.0,  1.0,  1.0,
        -1.0,  1.0, -1.0
    ];

    let position_buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));
    unsafe {
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&vertices),
            WebGlRenderingContext::STATIC_DRAW
        );
    }
  
    let face_colors = [
        [1.0,  1.0,  1.0,  1.0],    // 前面: 白
        [1.0,  0.0,  0.0,  1.0],    // 背面: 赤
        [0.0,  1.0,  0.0,  1.0],    // 上面: 緑
        [0.0,  0.0,  1.0,  1.0],    // 底面: 青
        [1.0,  1.0,  0.0,  1.0],    // 右側面: 黄
        [1.0,  0.0,  1.0,  1.0]     // 左側面: 紫
    ];
    let colors: Vec<f32> = face_colors.into_iter().flat_map(|cg| {
        (0..4).into_iter().flat_map(|_| cg).map(|c| *c).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let position_color_buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_color_buffer));
    unsafe {
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&colors),
            WebGlRenderingContext::STATIC_DRAW
        );
    }

    let cube_vertices_index_buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&cube_vertices_index_buffer));

    // この配列はそれぞれの面を 2 つの三角形として定義しており、
    // 各三角形の位置を指定するために、頂点の配列を指し示す
    // インデックスを使用します。
    let cube_vertex_indices = [
        0,  1,  2,      0,  2,  3,    // 前面
        4,  5,  6,      4,  6,  7,    // 背面
        8,  9,  10,     8,  10, 11,   // 上面
        12, 13, 14,     12, 14, 15,   // 底面
        16, 17, 18,     16, 18, 19,   // 右側面
        20, 21, 22,     20, 22, 23    // 左側面
    ];
    unsafe {
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &js_sys::Uint16Array::view(&cube_vertex_indices),
            WebGlRenderingContext::STATIC_DRAW
        );
    }

    let cube_vertices_normal_buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&cube_vertices_normal_buffer));

    // 頂点の法線ベクトル
    let vertex_normals = [
        // 前面
        0.0,  0.0,  1.0,
        0.0,  0.0,  1.0,
        0.0,  0.0,  1.0,
        0.0,  0.0,  1.0,

        // 背面
        0.0,  0.0, -1.0,
        0.0,  0.0, -1.0,
        0.0,  0.0, -1.0,
        0.0,  0.0, -1.0,

        // 上面
        0.0,  1.0,  0.0,
        0.0,  1.0,  0.0,
        0.0,  1.0,  0.0,
        0.0,  1.0,  0.0,

        // 底面
        0.0, -1.0,  0.0,
        0.0, -1.0,  0.0,
        0.0, -1.0,  0.0,
        0.0, -1.0,  0.0,

        // 右側面
        1.0,  0.0,  0.0,
        1.0,  0.0,  0.0,
        1.0,  0.0,  0.0,
        1.0,  0.0,  0.0,

        // 左側面
       -1.0,  0.0,  0.0,
       -1.0,  0.0,  0.0,
       -1.0,  0.0,  0.0,
       -1.0,  0.0,  0.0
    ];
    unsafe {
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&vertex_normals),
            WebGlRenderingContext::STATIC_DRAW
        )
    }

    (position_buffer, cube_vertices_index_buffer, cube_vertices_normal_buffer)
}

fn draw_scene(
    context: &WebGlRenderingContext,
    shader_program: &WebGlProgram,
    vertex_position: u32,
    vertex_normal: u32,
    program_projection_matrix: &WebGlUniformLocation,
    program_model_view_matrix: &WebGlUniformLocation,
    program_normal_matrix: &WebGlUniformLocation,
    position_buffer: &WebGlBuffer,
    cube_vertices_index_buffer: &WebGlBuffer,
    cube_vertices_normal_buffer: &WebGlBuffer,
    start_time: f64,
    current_time: f64
) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.enable(WebGlRenderingContext::DEPTH_TEST);
    context.depth_func(WebGlRenderingContext::LEQUAL);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

    let canvas = context.canvas().unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
    let aspect = canvas.client_width() as f32 / canvas.client_height() as f32;
    let z_near = 0.1;
    let z_far = 100.0;

    let projection_matrix = glm::perspective(aspect, field_of_view, z_near, z_far);
    let vec_projection_matrix = projection_matrix.iter().map(|v| *v).collect::<Vec<_>>();

    let delta = (current_time - start_time) as f32;
    let model_view_matrix = glm::translate(&glm::Mat4::identity(), &glm::TVec3::new(-0.0, 0.0, -6.0));
    let model_view_matrix = glm::rotate(&model_view_matrix, delta, &glm::TVec3::new(0.0, 0.0, 1.0));
    let model_view_matrix = glm::rotate(&model_view_matrix, delta*0.7, &glm::TVec3::new(0.0, 1.0, 0.0));
    let vec_model_view_matrix = model_view_matrix.iter().map(|v| *v).collect::<Vec<_>>();

    let normal_matrix = glm::transpose(&glm::inverse(&model_view_matrix));
    let vec_normal_matrix = normal_matrix.iter().map(|v| *v).collect::<Vec<_>>();

    {
        let num_components = 3;
        let data_type: u32 = WebGlRenderingContext::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));
        context.vertex_attrib_pointer_with_i32(
            vertex_position,
            num_components,
            data_type,
            normalize,
            stride,
            offset
        );
        context.enable_vertex_attrib_array(vertex_position);
    }

    {
        let num_components = 3;
        let data_type: u32 = WebGlRenderingContext::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&cube_vertices_normal_buffer));
        context.vertex_attrib_pointer_with_i32(
            vertex_normal,
            num_components,
            data_type,
            normalize,
            stride,
            offset
        );
        context.enable_vertex_attrib_array(vertex_normal);
    }

    {
        context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&cube_vertices_index_buffer))
    }

    context.use_program(Some(&shader_program));

    context.uniform_matrix4fv_with_f32_array(
        Some(program_projection_matrix),
        false,
        &vec_projection_matrix
    );

    context.uniform_matrix4fv_with_f32_array(
        Some(program_model_view_matrix),
        false,
        &vec_model_view_matrix
    );

    context.uniform_matrix4fv_with_f32_array(
        Some(program_normal_matrix),
        false,
        &vec_normal_matrix
    );

    let offset = 0;
    let vertex_count = 36;
    let data_type = WebGlRenderingContext::UNSIGNED_SHORT; // bufferの型UInt32Arrayに対応
    context.draw_elements_with_i32(WebGlRenderingContext::TRIANGLES, vertex_count, data_type, offset);

    // log(&format!("vertex position: {:?}", vertex_position));
    // log(&format!("projection matrix: \n{}", format_as_matrix(vec_projection_matrix, 4, 4)));
    // log(&format!("program projection matrix: {:?}", program_projection_matrix));
    // log(&format!("model view matrix: \n{}", format_as_matrix(vec_model_view_matrix, 4, 4)));
    // log(&format!("program model view matrix: {:?}", program_model_view_matrix));
}

// fn format_as_matrix<T: std::fmt::Display>(vec: Vec<T>, len_row: usize, len_column: usize) -> String {
//     let len = vec.len();
//     if len != len_column * len_row {
//         panic!("vector couldn't be divided by len_row");
//     }

//     (0..len_row).into_iter().map(|i| {
//         (0..len_column).into_iter().map(|j| {
//             format!("{}", &vec[i*len_row+j])
//         }).collect::<Vec<_>>().join(",")
//     }).collect::<Vec<_>>().join("\n")
// }

fn get_current_time() -> f64 { // sec
    js_sys::Date::now() / 1000.0
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

