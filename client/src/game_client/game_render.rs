use ipg_core::game::{map::Map, Galaxy, Game, Move, Planet};
use std::rc::Rc;
use std::{f64::consts::PI, marker::PhantomData};
use wasm_bindgen::{JsCast, JsValue};
// use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
};

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

macro_rules! check_webgl {
    ($t:expr) => {
        #[cfg(feature = "webgl_errors")]
        {
            if $t.get_error() != 0 {
                return Err(format!("WebGL Operation failed {}", $t.get_error()));
            }
        }
    };
}

pub struct GameRender {
    canvases: [HtmlCanvasElement; 2],
    gl: Rc<WebGl2RenderingContext>,
    ship_shader: Rc<WebGlProgram>,
    context_2d: CanvasRenderingContext2d,
    completed_move_index: usize,
    move_renders: Vec<MoveRender>,
    selection_render: SelectionRender,
}

// Replicated in ts\conts.ts
// Be sure to update it there as well
pub static PLAYER_COLORS: [&str; 9] = [
    "#878787", //Neutral Gray
    "#de4b37", //Red
    "#7dc740", //Green
    "#40c78f", //Teal
    "#30a1c7", //Blue
    "#3033c7", //Deep Blue
    "#d65cd2", //Magenta
    "#e0dc65", //Yellow
    "#a565e0", //Lavender
];

static SHIP_VERTEX: &'static str = concat!(
    r#"#version 300 es

#ifdef GL_ES
precision mediump float;
#endif

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 start_pos;
out float arrived;
uniform uint travel_time;
uniform vec2 destination;
uniform uint res_x;
uniform uint res_y;
uniform float from_radius;
uniform float to_radius;
void main()
{
    vec2 dir_norm = normalize(destination - start_pos);
    mat2 transform = mat2(
        dir_norm.y, dir_norm.x,
        -1.0 * dir_norm.x, dir_norm.y
    );

    vec2 ratio = vec2(float(res_x),float(res_y));
    float dist = distance(start_pos,destination);
    float remaining_dist = dist - (0.5 * float(travel_time));
    arrived = to_radius - remaining_dist;
    float progress = remaining_dist / dist;
    vec2 travel = mix(destination, start_pos, progress);
    gl_Position = vec4(pos * transform + (travel / ratio * 2.0 - 1.0), 0.0, 1.0);
    //gl_Position = vec4(pos, 1.0, 1.0);
}  
"#
);

static SHIP_FRAGMENT: &'static str = r#"#version 300 es
#ifdef GL_ES
precision mediump float;
#endif

layout(location = 0) out vec4 ship_color;
in float arrived; // Is positive when ship has arrived.

void main()
{
    if (arrived > 0.0) {
        ship_color = vec4(0.0,0.0,0.0,0.0);
    } else {
        ship_color = vec4(0.8,0.8,0.8,1.0);
    }
}

"#;

static SELECTION_VERTEX: &'static str = r#"#version 300 es
#ifdef GL_ES
precision mediump float;
#endif

layout (location = 0) in vec2 pos;
void main()
{
    gl_Position = vec4(pos, 1.0, 1.0);
}
"#;

static SELECTION_FRAGMENT: &'static str = r#"#version 300 es
#ifdef GL_ES
precision mediump float;
#endif
#define PI 3.1415926538

#define RING_WIDTH 8.0
#define RING_GAP 4.0
layout(location = 0) out vec4 color;
uniform vec2 center;
uniform float radius;
uniform vec2 dimensions;

void main()
{
    float ringSize = radius + RING_WIDTH +  RING_GAP;
    vec2 pixelCoord = gl_FragCoord.xy;
    float dist = distance(vec2(center.x,dimensions.y - center.y),pixelCoord);
    if ((dist > ringSize) || (dist < ringSize - RING_WIDTH)) {
        color = vec4(0.0,0.0,0.0,0.0);
    } else {
        color = vec4(0.0,1.0,1.0,0.5);
    }
}
"#;

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);
    context.validate_program(&program);
    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn create_ship_shader(gl_context: &WebGl2RenderingContext) -> Result<WebGlProgram, JsValue> {
    let vertex_shader = compile_shader(
        gl_context,
        WebGl2RenderingContext::VERTEX_SHADER,
        SHIP_VERTEX,
    )?;
    let fragment_shader = compile_shader(
        gl_context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        SHIP_FRAGMENT,
    )?;
    let program = link_program(gl_context, &vertex_shader, &fragment_shader)?;
    Ok(program)
}

pub fn render_map(
    context_2d: &CanvasRenderingContext2d,
    map: &Map,
    player_count: usize,
    width: u32,
    height: u32,
) -> Result<(), JsValue> {
    context_2d.clear_rect(0f64, 0f64, width as f64, height as f64);
    let x_ratio = width as f64 / map.size.x as f64;
    let y_ratio = height as f64 / map.size.y as f64;
    let ratio = y_ratio.min(x_ratio);
    for planet in &map.planets {
        context_2d.begin_path();
        context_2d.arc(
            planet.x as f64 * ratio,
            planet.y as f64 * ratio,
            f64::from(planet.radius) * ratio,
            0f64,
            2f64 * PI,
        )?;
        context_2d.set_fill_style(&JsValue::from(
            PLAYER_COLORS[planet.possession[player_count] as usize],
        ));
        context_2d.fill();
        context_2d.set_fill_style(&JsValue::from("#ffffff"));
        context_2d.set_font(&"20px Sans-Serif");
        context_2d.set_text_align(&"center");
        context_2d.set_text_baseline(&"middle");
        context_2d.fill_text(
            (planet.start_value as u32).to_string().as_str(),
            planet.x as f64 * ratio,
            planet.y as f64 * ratio,
        )?;
    }
    Ok(())
}

impl GameRender {
    pub fn new(
        canvas_top: HtmlCanvasElement,
        canvas_bottom: HtmlCanvasElement,
    ) -> Result<Self, JsValue> {
        let gl_context = canvas_top
            .get_context("webgl2")?
            .expect("Unwrap gl_context")
            .dyn_into::<WebGl2RenderingContext>()?;

        let context_2d = canvas_bottom
            .get_context("2d")?
            .expect("Unwrap 2d context")
            .dyn_into::<CanvasRenderingContext2d>()?;
        let ship_shader = create_ship_shader(&gl_context)?;
        let gl_ctx = Rc::new(gl_context);
        let ship_shader = Rc::new(ship_shader);
        Ok(Self {
            canvases: [canvas_top, canvas_bottom],
            gl: gl_ctx.clone(),
            ship_shader,
            context_2d,
            completed_move_index: 0,
            move_renders: Vec::new(),
            selection_render: SelectionRender::new(gl_ctx)?,
        })
    }

    pub fn render(&mut self, game: &Game, selected_planet: &Option<Planet>) -> Result<(), JsValue> {
        self.render_galaxy(game)?;
        if let Some(state) = &game.state {
            self.setup_gl_render()?;
            self.render_ships(state, &game.map)?;
            if let Some(selection) = selected_planet {
                self.render_selection(selection)?;
            }
        };
        Ok(())
    }

    pub fn setup_gl_render(&self) -> Result<(), String> {
        self.gl.cull_face(WebGl2RenderingContext::FRONT_AND_BACK);
        self.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        check_webgl!(self.gl);
        self.gl.viewport(
            0,
            0,
            self.canvases[0].width() as i32,
            self.canvases[0].height() as i32,
        );
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        check_webgl!(self.gl);
        Ok(())
    }

    pub fn render_galaxy(&mut self, game: &Game) -> Result<(), JsValue> {
        for canvas in &self.canvases {
            canvas.set_width(game.map.size.x);
            canvas.set_height(game.map.size.y);
        }
        match &game.state {
            Some(state) => {
                for planet in &state.planets {
                    self.context_2d.begin_path();
                    self.context_2d.arc(
                        planet.x as f64,
                        planet.y as f64,
                        planet.radius.into(),
                        0f64,
                        2f64 * PI,
                    )?;
                    self.context_2d.set_fill_style(&JsValue::from(
                        PLAYER_COLORS[planet.possession.map(|p| p + 1).unwrap_or(0)],
                    ));
                    self.context_2d.fill();
                    self.context_2d.set_fill_style(&JsValue::from("#ffffff"));
                    self.context_2d.set_font(&"20px Sans-Serif");
                    self.context_2d.set_text_align(&"center");
                    self.context_2d.set_text_baseline(&"middle");
                    self.context_2d.fill_text(
                        (planet.value as u32).to_string().as_str(),
                        planet.x as f64,
                        planet.y as f64,
                    )?;
                }
            }
            None => {
                render_map(
                    &self.context_2d,
                    &game.map,
                    game.players.len(),
                    game.map.size.x,
                    game.map.size.y,
                )?;
            }
        };
        Ok(())
    }

    pub fn render_selection(&mut self, selected_planet: &Planet) -> Result<(), String> {
        self.selection_render.set_size(
            self.canvases[0].width() as i32,
            self.canvases[0].height() as i32,
        );
        self.selection_render.render(selected_planet);
        Ok(())
    }

    pub fn render_ships(&mut self, galaxy: &Galaxy, map: &Map) -> Result<(), String> {
        self.gl.use_program(Some(&self.ship_shader));
        check_webgl!(self.gl);
        for game_move in (&galaxy.moves)
            .iter()
            .skip(self.completed_move_index)
            .filter(|game_move| game_move.end_time() > galaxy.time)
        {
            self.move_renders.push(MoveRender::new(
                game_move.clone(),
                self.gl.clone(),
                self.ship_shader.clone(),
                map,
            )?);
            self.completed_move_index += 1;
        }
        self.move_renders
            .retain(|move_render| move_render.game_move.end_time() > galaxy.time);
        for move_render in self.move_renders.iter() {
            move_render.render(&galaxy, map)?;
        }
        Ok(())
    }
}

static SHIP_START_POS: u32 = 1;
// static SHIP_START_TIME: u32 = 2;
// static SHIP_TRAVEL_TIME: u32 = 2;
static SHIP_VERTS: u32 = 0;

pub struct MoveRender {
    game_move: Move,
    gl: Rc<WebGl2RenderingContext>,
    program: Rc<WebGlProgram>,
    // These fields are needed to prevent the buffer from being dropped (which will cause them to be cleaned up)
    // before the MoveRender is dropped
    positions_vbo: Buffer<f32>,
    verts_vbo: Buffer<f32>,
    verts_vao: web_sys::WebGlVertexArrayObject,
}

impl MoveRender {
    pub fn new(
        game_move: Move,
        gl_ctx: Rc<WebGl2RenderingContext>,
        program: Rc<WebGlProgram>,
        map: &Map,
    ) -> Result<MoveRender, String> {
        let ship_count = game_move.armada_size;
        let mut positions = vec![0f32; ship_count as usize * 2];
        {
            let mut i = 0usize;
            for (x, y) in game_move.start_positions() {
                positions[i] = x;
                i += 1;
                positions[i] = map.size.y as f32 - y;
                i += 1; //Flip y axis
            }
        }

        //VAO
        let vao = gl_ctx
            .create_vertex_array()
            .ok_or("Could not vertex array")?;
        check_webgl!(gl_ctx);

        gl_ctx.bind_vertex_array(Some(&vao));
        check_webgl!(gl_ctx);

        //Ship Positions
        let ship_positions_vbo = Buffer::new(positions, gl_ctx.clone())?;

        // Ship Verticies
        let ship_verts: Vec<f32> = vec![-0.25f32, -0.5, 0.25, -0.5, 0.0, 0.5]
            .iter()
            .map(|f| f / 80.0)
            .collect();
        let ship_verts_vbo = Buffer::new(ship_verts, gl_ctx.clone())?;

        // Current WIP: Moving to the new buffer abstraction

        gl_ctx.enable_vertex_attrib_array(SHIP_VERTS);
        check_webgl!(gl_ctx);

        gl_ctx.vertex_attrib_pointer_with_i32(
            SHIP_VERTS,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        // Add vertex & position information to the VAO
        gl_ctx.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&ship_positions_vbo.get_buf()),
        );
        gl_ctx.vertex_attrib_pointer_with_i32(
            SHIP_START_POS,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        check_webgl!(gl_ctx);
        gl_ctx.enable_vertex_attrib_array(SHIP_START_POS);
        check_webgl!(gl_ctx);
        gl_ctx.vertex_attrib_divisor(SHIP_START_POS, 1);
        check_webgl!(gl_ctx);
        Ok(MoveRender {
            game_move,
            gl: gl_ctx,
            program,
            positions_vbo: ship_positions_vbo,
            verts_vbo: ship_verts_vbo,
            verts_vao: vao,
        })
    }

    pub fn render(&self, galaxy: &Galaxy, map: &Map) -> Result<(), String> {
        self.gl.bind_vertex_array(Some(&self.verts_vao));
        // Uniforms
        if let Some(travel_time_loc) = self.gl.get_uniform_location(&self.program, "travel_time") {
            self.gl.uniform1ui(
                Some(&travel_time_loc),
                galaxy.time - self.game_move.start_time,
            );
        } else {
            log!("WARNING: Unable to find uniform travel_time.");
        };
        check_webgl!(self.gl);

        // These uniforms need to be set every render (because we re-use the program between move renders)
        // Ideally the "get_uniform_location" calls should be moved out of the render loop though

        let destination_loc = self
            .gl
            .get_uniform_location(&self.program, "destination")
            .ok_or("Unable to find uniform.")?;
        self.gl.uniform2fv_with_f32_array(
            Some(&destination_loc),
            vec![
                self.game_move.to.x as f32,
                (map.size.y as f32) - self.game_move.to.y as f32,
            ]
            .as_slice(),
        );
        if let Some(res_x) = self.gl.get_uniform_location(&self.program, "res_x") {
            self.gl.uniform1ui(Some(&res_x), map.size.x);
        } else {
            log!("WARNING: Unable to find uniform res_x.");
        };
        if let Some(res_y) = self.gl.get_uniform_location(&self.program, "res_y") {
            self.gl.uniform1ui(Some(&res_y), map.size.y);
        } else {
            log!("WARNING: Unable to find uniform res_y.");
        };
        if let Some(to_radius) = self.gl.get_uniform_location(&self.program, "to_radius") {
            self.gl
                .uniform1f(Some(&to_radius), self.game_move.to.radius);
        } else {
            log!("WARNING: Unable to find uniform to_radius.");
        };
        if let Some(from_radius) = self.gl.get_uniform_location(&self.program, "from_radius") {
            self.gl
                .uniform1f(Some(&from_radius), self.game_move.from.radius);
        } else {
            log!("WARNING: Unable to find uniform from_radius.");
        };

        // Draw
        self.gl.draw_arrays_instanced(
            WebGl2RenderingContext::TRIANGLES,
            0,
            3,
            self.game_move.armada_size as i32,
        );
        //log!("ship_count: {}, start_times: {:?}, positions: {:?}", ship_count, start_times, positions);
        check_webgl!(self.gl);
        Ok(())
    }
}

impl Drop for MoveRender {
    fn drop(&mut self) {
        log!("dropping move at {}", self.game_move.start_time);
        //Cleanup
        self.gl.delete_vertex_array(Some(&self.verts_vao));
    }
}

struct SelectionRender {
    gl: Rc<WebGl2RenderingContext>,
    program: WebGlProgram,
    vao: WebGlVertexArrayObject,
    _verts_vbo: Buffer<f32>,
}

impl SelectionRender {
    pub fn new(gl_ctx: Rc<WebGl2RenderingContext>) -> Result<Self, String> {
        // Positions
        let square_verts: Vec<f32> = vec![
            -1.0, -1.0, //
            1.0, -1.0, //
            -1.0, 1.0, //
            //
            1.0, -1.0, //
            -1.0, 1.0, //
            1.0, 1.0, //
        ];
        let verts_vbo = Buffer::new(square_verts, gl_ctx.clone())?;

        // Program
        let program = SelectionRender::create_shader(&*gl_ctx)?;

        //VAO
        let vao = gl_ctx
            .create_vertex_array()
            .ok_or("Could not vertex array")?;

        gl_ctx.bind_vertex_array(Some(&vao));
        check_webgl!(gl_ctx);

        // Add vertex information to the VAO
        const SELECT_VERTS: u32 = 0;

        gl_ctx.enable_vertex_attrib_array(SELECT_VERTS);

        gl_ctx.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&verts_vbo.get_buf()),
        );
        gl_ctx.vertex_attrib_pointer_with_i32(
            SELECT_VERTS,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        Ok(SelectionRender {
            gl: gl_ctx,
            program,
            vao,
            _verts_vbo: verts_vbo,
        })
    }

    pub fn set_size(&self, x: i32, y: i32) {
        self.gl.use_program(Some(&self.program));
        let dimensions_uniform = self.gl.get_uniform_location(&self.program, "dimensions");
        self.gl
            .uniform2f(dimensions_uniform.as_ref(), x as f32, y as f32);
    }

    fn create_shader(gl_ctx: &WebGl2RenderingContext) -> Result<WebGlProgram, String> {
        let vertex_shader = compile_shader(
            gl_ctx,
            WebGl2RenderingContext::VERTEX_SHADER,
            SELECTION_VERTEX,
        )?;
        let fragment_shader = compile_shader(
            gl_ctx,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            SELECTION_FRAGMENT,
        )?;
        let program = link_program(gl_ctx, &vertex_shader, &fragment_shader)?;
        Ok(program)
    }

    fn render(&self, selected_planet: &Planet) {
        self.gl.use_program(Some(&self.program));
        self.gl.bind_vertex_array(Some(&self.vao));
        // Set Uniforms
        let center_uniform = self.gl.get_uniform_location(&self.program, "center");
        if center_uniform.is_none() {
            log!("WARNING: Unable to find uniform center.");
        }
        self.gl.uniform2f(
            center_uniform.as_ref(),
            selected_planet.x as f32,
            selected_planet.y as f32,
        );
        let radius_uniform = self.gl.get_uniform_location(&self.program, "radius");
        if radius_uniform.is_none() {
            log!("WARNING: Unable to find uniform radius.");
        }
        self.gl
            .uniform1f(radius_uniform.as_ref(), selected_planet.radius as f32);
        // Draw
        self.gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            6, // This should be the number of verts in the data defined in the constructor
        );
    }
}

impl Drop for SelectionRender {
    fn drop(&mut self) {
        self.gl.delete_vertex_array(Some(&self.vao));
        self.gl.delete_program(Some(&self.program));
    }
}

struct Buffer<T> {
    buf: web_sys::WebGlBuffer,
    gl: Rc<WebGl2RenderingContext>,
    _phantom: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn get_buf(&self) -> &web_sys::WebGlBuffer {
        return &self.buf;
    }
    pub fn new(data: Vec<T>, gl_ctx: Rc<WebGl2RenderingContext>) -> Result<Self, String> {
        let buffer_obj = gl_ctx.create_buffer().ok_or("Could not create buffer")?;
        gl_ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer_obj));
        check_webgl!(gl_ctx);
        unsafe {
            gl_ctx.buffer_data_with_u8_array(
                WebGl2RenderingContext::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    data.as_ptr() as *const u8,
                    data.len() * std::mem::size_of::<T>(),
                ),
                WebGl2RenderingContext::STATIC_DRAW,
            );
            check_webgl!(gl_ctx);
        }

        Ok(Buffer {
            buf: buffer_obj,
            gl: gl_ctx,
            _phantom: PhantomData,
        })
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        //Cleanup
        self.gl.delete_buffer(Some(&self.buf));
    }
}
