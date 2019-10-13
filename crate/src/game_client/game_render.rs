use web_sys::{ HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, CanvasRenderingContext2d, WebGlShader };
use wasm_bindgen::{ JsCast, JsValue };
use ipg_core::game::{ Galaxy, Game };
use std::f64::consts::PI;
use web_sys::console;

pub struct GameRender {
    canvas: HtmlCanvasElement,
    //gl_context: WebGlRenderingContext,
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

static SHIP_MOVEMENT_SHADER: &'static str = r#"

"#; 

static SHIP_VERTEX: &'static str = r#"

"#; 

static SHIP_FRAGMENT: &'static str = r#"

"#; 

fn setup_canvas(canvas: &mut WebGlRenderingContext) {
    
}

impl GameRender {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // let gl_context = canvas
        // .get_context("webgl")?
        // .expect("Unwrap gl_context")
        // .dyn_into::<WebGlRenderingContext>()?;

        let context_2d = canvas
        .get_context("2d")?
        .expect("Unwrap 2d context")
        .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Self {
            canvas,
            //gl_context,
            context_2d
        })
    }

    pub fn render_galaxy(&mut self, game: &Game) {
        self.canvas.set_width(game.map.size.x);
        self.canvas.set_height(game.map.size.y);
        for planet in &game.map.planets {
            self.context_2d.begin_path();
            self.context_2d.arc(planet.x as f64, planet.y as f64, planet.radius.into(), 0f64, 2f64 * PI);
            self.context_2d.set_stroke_style(&JsValue::from(PLAYER_COLORS[planet.possession[game.players.len()] as usize]));
            self.context_2d.stroke();
        }
        match &game.state {
            Some(state) => {
                for planet in &state.planets {
                    self.context_2d.fill_text((planet.value as u32).to_string().as_str(),planet.x as f64,planet.y as f64);
                }
            },
            None => {
                for planet in &game.map.planets {
                    self.context_2d.fill_text((planet.start_value as u32).to_string().as_str(),planet.x as f64,planet.y as f64);
                }
            }
        };
    }
}