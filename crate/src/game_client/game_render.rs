use web_sys::{ HtmlCanvasElement, WebGlProgram, WebGl2RenderingContext, CanvasRenderingContext2d, WebGlShader };
use wasm_bindgen::{ JsCast, JsValue };
use ipg_core::game::{ Galaxy, Game, Move, map::Map, SHIP_SPEED };
use std::f64::consts::PI;
use web_sys::console;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

macro_rules! check_webgl {
    ($t:expr) => {
        if $t.get_error() != 0 {
            return Err(format!("WebGL Operation failed {}",$t.get_error()));
        };
    };
}

pub struct GameRender {
    canvases: [HtmlCanvasElement;2],
    gl: WebGl2RenderingContext,
    context_2d: CanvasRenderingContext2d,
}

static PLAYER_COLORS: [&str;9] = [
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

static SHIP_VERTEX: &'static str = concat!(r#"#version 300 es

#ifdef GL_ES
precision mediump float;
#endif

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 start_pos;
uniform uint travel_time;
uniform vec2 destination;

void main()
{
    float dist = distance(start_pos,destination) * 1000.0;
    float progress = (dist - (0.5 * float(travel_time))) / dist;
    vec2 travel = mix(destination, start_pos, progress);
    gl_Position = vec4(pos + travel, 0.0, 1.0);
    //gl_Position = vec4(pos, 1.0, 1.0);
}  
"#); 

static SHIP_FRAGMENT: &'static str = r#"#version 300 es
#ifdef GL_ES
precision mediump float;
#endif

layout(location = 0) out vec4 ship_color;

void main()
{
    ship_color = vec4(1.0,0.0,1.0,1.0);
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

impl GameRender {
    pub fn new(canvas_top: HtmlCanvasElement,canvas_bottom: HtmlCanvasElement) -> Result<Self, JsValue> {
        let gl_context = canvas_top
        .get_context("webgl2")?
        .expect("Unwrap gl_context")
        .dyn_into::<WebGl2RenderingContext>()?;

        let context_2d = canvas_bottom
        .get_context("2d")?
        .expect("Unwrap 2d context")
        .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Self {
            canvases: [canvas_top,canvas_bottom],
            gl: gl_context,
            context_2d
        })
    }

    pub fn render_galaxy(&mut self, game: &Game) -> Result<(),JsValue> {
        for canvas in &self.canvases {
            canvas.set_width(game.map.size.x);
            canvas.set_height(game.map.size.y);
        };
        match &game.state {
            Some(state) => {
                for planet in &state.planets {
                    self.context_2d.begin_path();
                    self.context_2d.arc(planet.x as f64, planet.y as f64, planet.radius.into(), 0f64, 2f64 * PI)?;
                    self.context_2d.set_stroke_style(&JsValue::from(PLAYER_COLORS[planet.possession.map(|p| p + 1).unwrap_or(0)]));
                    self.context_2d.stroke();
                    self.context_2d.fill_text((planet.value as u32).to_string().as_str(),planet.x as f64,planet.y as f64)?;
                };
                self.render_ships(state,&game.map)?;
            },
            None => {
                for planet in &game.map.planets {
                    self.context_2d.begin_path();
                    self.context_2d.arc(planet.x as f64, planet.y as f64, planet.radius.into(), 0f64, 2f64 * PI)?;
                    self.context_2d.set_stroke_style(&JsValue::from(PLAYER_COLORS[planet.possession[game.players.len()] as usize]));
                    self.context_2d.stroke();
                    self.context_2d.fill_text((planet.start_value as u32).to_string().as_str(),planet.x as f64,planet.y as f64)?;
                }
            }
        };
        Ok(())
    }

    pub fn render_ships(&mut self, galaxy: &Galaxy, map: &Map) -> Result<(),String> {
        log!("render_ships");
        self.gl.cull_face(WebGl2RenderingContext::FRONT_AND_BACK);
        self.gl.viewport(0,0,self.canvases[0].width() as i32,self.canvases[0].height() as i32);

        //Shader Setup
        let vertex_shader = compile_shader(&self.gl,WebGl2RenderingContext::VERTEX_SHADER,SHIP_VERTEX)?;
        let fragment_shader = compile_shader(&self.gl,WebGl2RenderingContext::FRAGMENT_SHADER,SHIP_FRAGMENT)?;
        let program = link_program(&self.gl,&vertex_shader,&fragment_shader)?;
        self.gl.use_program(Some(&program));
        check_webgl!(self.gl);

        for game_move in (&galaxy.moves).iter().filter(|game_move| game_move.end_time() > galaxy.time) {
            log!("Processing move!");
            let ship_count = game_move.armada_size;
            let mut positions = vec![0f32; ship_count as usize * 2];
            { 
                let mut i = 0usize;
                for (x,y) in game_move.start_positions() {
                    positions[i] = x / (map.size.x as f32) * 2.0 - 1.0 ; i += 1;
                    positions[i] = (y / (map.size.y as f32) * 2.0 - 1.0) * - 1.0; i += 1;
                }
            }

            //VAO
            let vao = self.gl.create_vertex_array().ok_or("Could not vertex array")?;
            check_webgl!(self.gl);

            self.gl.bind_vertex_array(Some(&vao));
            check_webgl!(self.gl);

            let ship_positions_vbo = self.gl.create_buffer().ok_or("Could not create buffer")?;
            self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER,Some(&ship_positions_vbo));
            unsafe {
                self.gl.buffer_data_with_u8_array(WebGl2RenderingContext::ARRAY_BUFFER, std::slice::from_raw_parts(positions.as_ptr() as *const u8, positions.len() * 4), WebGl2RenderingContext::STATIC_DRAW);
            }
            self.gl.vertex_attrib_pointer_with_i32(SHIP_START_POS,2,WebGl2RenderingContext::FLOAT, false, 0, 0);
            check_webgl!(self.gl);
            self.gl.enable_vertex_attrib_array(SHIP_START_POS);
            check_webgl!(self.gl);
            self.gl.vertex_attrib_divisor(SHIP_START_POS,1);
            check_webgl!(self.gl);
            //self.gl.bind_attrib_location(&program,SHIP_START_POS,"start_pos");
            //check_webgl!(self.gl);

            //Start Times
                // //Start Times
                // let mut start_times = vec![0u16; ship_count as usize];
                // for game_move in (&galaxy.moves).iter().filter(|game_move| game_move.end_time() < galaxy.time) {
                //     for i in 0..(game_move.armada_size as usize) {
                //         start_times[i] = game_move.time as u16;
                //     }
                // }

                // let start_times_vbo = self.gl.create_buffer();
                // check_webgl!(self.gl);
                // self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER,start_times_vbo.as_ref());
                // check_webgl!(self.gl);
                // unsafe {
                //     self.gl.buffer_data_with_u8_array(WebGl2RenderingContext::ARRAY_BUFFER, std::slice::from_raw_parts(start_times.as_ptr() as *const u8, start_times.len() * 2), WebGl2RenderingContext::STATIC_DRAW);
                //     check_webgl!(self.gl);
                // }
                // self.gl.enable_vertex_attrib_array(SHIP_START_TIME);
                // check_webgl!(self.gl);
                // self.gl.vertex_attrib_pointer_with_i32(SHIP_START_TIME,2,WebGl2RenderingContext::UNSIGNED_SHORT, false, 0, 0);
                // check_webgl!(self.gl);
                // self.gl.bind_attrib_location(&program,SHIP_START_TIME,"start_time");
                // check_webgl!(self.gl);
            //

            // Uniforms
            if let Some(travel_time_loc) = self.gl.get_uniform_location(&program,"travel_time") {
                self.gl.uniform1ui(Some(&travel_time_loc),galaxy.time - game_move.time);
                log!("Travel time: {}", galaxy.time - game_move.time)
            } else {
                log!("WARNING: Unable to find uniform travel_time.");
            };
            let destination_loc = self.gl.get_uniform_location(&program,"destination").ok_or("Unable to find uniform.")?;
            self.gl.uniform2fv_with_f32_array(Some(&destination_loc),vec![
                game_move.to.x as f32 / (map.size.x as f32) * 2.0 - 1.0,
                game_move.to.y as f32 / (map.size.y as f32) * 2.0 - 1.0,
            ].as_slice());

            // Ship Verticies
            let ship_verts: Vec<f32> = vec![-0.25f32,-0.5,0.25,-0.5,0.0,0.5].iter().map(|f| f/80.0).collect();
            let ship_verts_vbo = self.gl.create_buffer().ok_or("Could not create buffer")?;
            self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER,Some(&ship_verts_vbo));
            check_webgl!(self.gl);
            unsafe {
                self.gl.buffer_data_with_u8_array(
                    WebGl2RenderingContext::ARRAY_BUFFER, 
                    std::slice::from_raw_parts(ship_verts.as_ptr() as *const u8, ship_verts.len() * 4),
                    WebGl2RenderingContext::STATIC_DRAW);
                check_webgl!(self.gl);
            }

            self.gl.enable_vertex_attrib_array(SHIP_VERTS);
            check_webgl!(self.gl);

            self.gl.vertex_attrib_pointer_with_i32(SHIP_VERTS,2,WebGl2RenderingContext::FLOAT, false, 0, 0);
            check_webgl!(self.gl);


            self.gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLES,0,3,ship_count as i32);
            //log!("ship_count: {}, start_times: {:?}, positions: {:?}", ship_count, start_times, positions);
            check_webgl!(self.gl);

            //Cleanup
            self.gl.delete_buffer(Some(&ship_positions_vbo));
            //self.gl.delete_buffer(start_times_vbo.as_ref());
            self.gl.delete_buffer(Some(&ship_verts_vbo));
            self.gl.delete_vertex_array(Some(&vao));


        };
        Ok(())
    }
}

static SHIP_START_POS: u32 = 1;
static SHIP_START_TIME: u32 = 2;
static SHIP_TRAVEL_TIME: u32 = 2;
static SHIP_VERTS: u32 = 0;