#![feature(phase)]
#![feature(unsafe_destructor)]

extern crate device;
#[phase(plugin)] extern crate gfx_macros;
extern crate getopts;
extern crate gfx;
extern crate glfw;
extern crate glfw_platform;
extern crate native;
extern crate render;
extern crate sync;

use glfw_platform::BuilderExtension;

#[vertex_format]
struct Vertex {
    pos: [f32, ..2],
    color: [f32, ..3],
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    in vec2 pos;
    in vec3 color;
    out vec4 v_Color;
    void main() {
        v_Color = vec4(color, 1.0);
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    in vec4 v_Color;
    out vec4 o_Color;
    void main() {
        o_Color = v_Color;
    }
"
};


struct GfxRsContext {
    glfw  : glfw::Glfw,
    window: glfw::Window,
    events: sync::comm::Receiver<(f64,glfw::WindowEvent)>,
    device: device::Device<render::resource::handle::Handle,device::gl::GlBackEnd,glfw_platform::Platform<glfw::RenderContext>>,
}


#[unsafe_destructor]
impl Drop for GfxRsContext {
    fn drop(&mut self) {
        println!("Dropping context!");
    }
}


impl GfxRsContext {
    fn init() -> GfxRsContext {
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        let (mut window, events) = glfw_platform::WindowBuilder::new(&glfw)
            .title("[GLFW] Triangle example #gfx-rs!")
            .try_modern_context_hints()
            .create()
            .expect("Failed to create GLFW window.");

        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
        window.set_key_polling(true); // so we can quit when Esc is pressed
        let (w, h) = window.get_size();

        let device = gfx::build()
            .with_glfw_window(&mut window)
            .with_queue_size(1)
            .spawn(proc(r) render_loop(r, w as u16, h as u16))
            .unwrap();

        let context = GfxRsContext {
            glfw  : glfw,
            window: window,
            events: events,
            device: device,
        };

        return context;
    }

    fn input(&self) -> bool {
        self.glfw.poll_events();
        if self.window.should_close() {
            return false;
        }
        // quit when Esc is pressed.
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => return false,
                _ => {},
            }
        }

        return true;
    }

    fn render(&mut self) {
        self.device.update();
    }
}

// We need to run on the main thread for GLFW, so ensure we are using the `native` runtime. This is
// technically not needed, since this is the default, but it's not guaranteed.
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
     native::start(argc, argv, main)
}

fn main() {
    let mut context = GfxRsContext::init();

    loop {
        if !context.input() {
            println!("Ok we're done here");
            drop(context);
            break;
        }
        context.render();
    }
}

fn render_loop(mut renderer: gfx::Renderer, width: u16, height: u16) {
    let frame = gfx::Frame::new(width, height);
    let state = gfx::DrawState::new();

    let vertex_data = vec![
        Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [ 0.5, -0.5 ], color: [0.0, 1.0, 0.0]  },
        Vertex { pos: [ 0.0, 0.5 ], color: [0.0, 0.0, 1.0]  }
    ];

    let mesh = renderer.create_mesh(vertex_data);
    let program = renderer.create_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone());

    let clear = gfx::ClearData {
        color: Some(gfx::Color([0.3, 0.3, 0.3, 1.0])),
        depth: None,
        stencil: None,
    };

    while !renderer.should_finish() {
        renderer.clear(clear, frame);
        renderer.draw(&mesh, mesh.get_slice(), &frame, &program, &state)
            .unwrap();
        println!("About to end a frame!");
        println!("Should we finish? {}", renderer.should_finish());
        renderer.end_frame();
        println!("Ended");
        for err in renderer.errors() {
            println!("Renderer error: {}", err);
        }
        std::io::timer::sleep(500);
    }
    println!("Renderer is finished");
}
