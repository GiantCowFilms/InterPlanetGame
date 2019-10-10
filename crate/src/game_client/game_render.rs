use web_sys::{ HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, CanvasRenderingContext2d, WebGlShader };
use wasm_bindgen::{ JsCast, JsValue };
use ipg_core::game::Galaxy;

const PI: f64 = 3.1415926535;

pub struct GameRender {
    gl_context: WebGlRenderingContext,
    context_2d: CanvasRenderingContext2d,
}

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
        let gl_context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

        let context_2d = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Self {
            gl_context,
            context_2d
        })
    }

    pub fn render_galaxy(&mut self, galaxy: &Galaxy) {
        for planet in &galaxy.planets {
            self.context_2d.begin_path();
            self.context_2d.arc(planet.x as f64, planet.y as f64, planet.radius.into(), 0f64, 2f64 * PI);
            self.context_2d.stroke();
            self.context_2d.fill_text((planet.value as u32).to_string().as_str(),planet.x as f64,planet.y as f64);
        }
    }
}