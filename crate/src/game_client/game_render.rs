use web_sys::{ HtmlCanvasElement, WebGlProgram, WebGl2RenderingContext, CanvasRenderingContext2d, WebGlShader };
use wasm_bindgen::{ JsCast, JsValue };
use ipg_core::game::{ Galaxy, Game, Move, map::Map };
use std::f64::consts::PI;
use web_sys::console;

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

static SHIP_VERTEX: &'static str = r#"
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 start_pos;
layout (location = 2) in uint start_time;

out vec3 fColor;

void main()
{
    gl_Position = vec4(pos + start_pos, 0.0, 1.0);
}  
"#; 

static SHIP_FRAGMENT: &'static str = r#"

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

    pub fn render_galaxy(&mut self, game: &Game) {
        for canvas in &self.canvases {
            canvas.set_width(game.map.size.x);
            canvas.set_height(game.map.size.y);
        };
        match &game.state {
            Some(state) => {
                for planet in &state.planets {
                    self.context_2d.begin_path();
                    self.context_2d.arc(planet.x as f64, planet.y as f64, planet.radius.into(), 0f64, 2f64 * PI);
                    self.context_2d.set_stroke_style(&JsValue::from(PLAYER_COLORS[planet.possession.map(|p| p + 1).unwrap_or(0)]));
                    self.context_2d.stroke();
                    self.context_2d.fill_text((planet.value as u32).to_string().as_str(),planet.x as f64,planet.y as f64);
                };
                self.render_ships(state,&game.map);
            },
            None => {
                for planet in &game.map.planets {
                    self.context_2d.begin_path();
                    self.context_2d.arc(planet.x as f64, planet.y as f64, planet.radius.into(), 0f64, 2f64 * PI);
                    self.context_2d.set_stroke_style(&JsValue::from(PLAYER_COLORS[planet.possession[game.players.len()] as usize]));
                    self.context_2d.stroke();
                    self.context_2d.fill_text((planet.start_value as u32).to_string().as_str(),planet.x as f64,planet.y as f64);
                }
            }
        };
    }

    pub fn render_ships(&mut self, galaxy: &Galaxy, map: &Map) -> Result<(),String> {
        //Shader Setup
        let vertex_shader = compile_shader(&self.gl,WebGl2RenderingContext::VERTEX_SHADER,SHIP_VERTEX)?;
        let fragment_shader = compile_shader(&self.gl,WebGl2RenderingContext::FRAGMENT_SHADER,SHIP_FRAGMENT)?;
        let program = link_program(&self.gl,&vertex_shader,&fragment_shader)?;
        //Ship Positions
        let ship_count = (&galaxy.moves).iter()
            .filter(|game_move| game_move.end_time() < galaxy.time)
            .fold(0,|sum,game_move| game_move.armada_size + sum);
        let mut positions = vec![0f32; ship_count as usize * 2];
        let mut i = 0usize;
        for game_move in (&galaxy.moves).iter().filter(|game_move| game_move.end_time() < galaxy.time) {
            for (x,y) in game_move.start_positions() {
                positions[i] = x / map.size.x as f32; i += 1;
                positions[i] = y / map.size.y as f32; i += 1;
            }
        }
        let vbo = self.gl.create_buffer();
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER,vbo.as_ref());
        unsafe {
            self.gl.buffer_data_with_u8_array(WebGl2RenderingContext::ARRAY_BUFFER, std::slice::from_raw_parts(positions.as_ptr() as *const u8, positions.len() * 4), WebGl2RenderingContext::STATIC_DRAW);
        }
        self.gl.vertex_attrib_pointer_with_i32(SHIP_START_POS,2,WebGl2RenderingContext::FLOAT, false, 0, 0);
        self.gl.bind_attrib_location(&program,SHIP_START_POS,"start_pos");
        //Start Times
        let mut start_times = vec![0u16; ship_count as usize];
        for game_move in (&galaxy.moves).iter().filter(|game_move| game_move.end_time() < galaxy.time) {
            for i in 0..(game_move.armada_size as usize) {
                start_times[i] = game_move.time as u16;
            }
        }
        let vbo = self.gl.create_buffer();
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER,vbo.as_ref());
        unsafe {
            self.gl.buffer_data_with_u8_array(WebGl2RenderingContext::ARRAY_BUFFER, std::slice::from_raw_parts(start_times.as_ptr() as *const u8, positions.len() * 2), WebGl2RenderingContext::STATIC_DRAW);
        }
        self.gl.vertex_attrib_pointer_with_i32(SHIP_START_POS,1,WebGl2RenderingContext::UNSIGNED_SHORT, false, 0, 0);
        self.gl.bind_attrib_location(&program,SHIP_START_POS,"start_time");

        self.gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLES,0,3,ship_count as i32);

        Ok(())
    }
}

static SHIP_START_POS: u32 = 0;