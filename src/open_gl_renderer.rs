#![allow(non_upper_case_globals)]
extern crate glfw;
use glfw::{Glfw, GlfwReceiver, PWindow, WindowEvent};
use rusty_ffmpeg::ffi::AVFrame;

use crate::consumer::Consumer;
use crate::sw_scale::ScalerOutput;
use crate::wrappers::WrappedAVFrame;

use self::glfw::{Context, Key, Action};

extern crate gl;
use self::gl::types::*;

use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::os::raw::c_void;
use std::sync::Mutex;

const vertexShaderSource: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;

    layout (location = 1) in vec3 aColor;
    layout (location = 2) in vec2 aTexCoord;

    out vec3 ourColor;
    out vec2 TexCoord;
    void main()
    {

        gl_Position = vec4(aPos, 1.0);
        ourColor = aColor;
        TexCoord = vec2(aTexCoord.x, aTexCoord.y);
    }
"#;

const fragmentShaderSource: &str = r#"
    #version 330 core
    out vec4 FragColor;

    in vec3 ourColor;
    in vec2 TexCoord;

    // texture sampler
    uniform sampler2D texture1;

    void main()
    {
        FragColor = texture(texture1, TexCoord);
    }
"#;

struct RenderThread{
    renderer: GLRenderer,
}

struct GLContext {
    glfw: Glfw,
    window: PWindow, 
    events: GlfwReceiver<(f64, WindowEvent)>,
    shader_program_id: u32,
    vao: u32,
    texture: u32,
    texture_initilized: bool
}
struct VideoContext {
    last_frame_pts: u64,
    last_frame: Option<WrappedAVFrame>,
    had_update: bool,
    frame_rate: u32
}
pub struct GLRenderer {
    width: u32,
    height: u32,
    gl_context: GLContext,
    video_context: Mutex<VideoContext>
}
impl GLRenderer {
    pub fn new(width: u32, height: u32, framerate: u32) -> Self {
        let gl_context = GLRenderer::init_gl(width, height);
        let video_context = Mutex::new(VideoContext { last_frame_pts: 0, last_frame: None, had_update: false, frame_rate: framerate});
        Self {
            width,
            height,
            gl_context,
            video_context,
        }
    }

    #[allow(non_snake_case)]
    fn init_gl (width: u32, height: u32) -> GLContext {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        let (mut window, events) = glfw.create_window(width, height, "Video Player", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        window.make_current();

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol).unwrap() as *const _);

        let (shaderProgram, VAO, texture) = unsafe {
            let shaderProgram = gl::CreateProgram();

            let (vertexShader, fragmentShader) = {
                let mut success = gl::FALSE as GLint;
                let mut infoLog = Vec::with_capacity(512);
                infoLog.set_len(512 - 1); // subtract 1 to skip the trailing null character
                                          //
                let c_str_vert = CString::new(vertexShaderSource.as_bytes()).unwrap();
                let vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
                gl::ShaderSource(vertexShader, 1, &c_str_vert.as_ptr(), ptr::null());
                gl::CompileShader(vertexShader);
                gl::GetShaderiv(vertexShader, gl::COMPILE_STATUS, &mut success);
                if success != gl::TRUE as GLint {
                    gl::GetShaderInfoLog(vertexShader, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
                    println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
                }

                let c_str_frag = CString::new(fragmentShaderSource.as_bytes()).unwrap();
                let fragmentShader = gl::CreateShader(gl::FRAGMENT_SHADER);
                gl::ShaderSource(fragmentShader, 1, &c_str_frag.as_ptr(), ptr::null());
                gl::CompileShader(fragmentShader);
                gl::GetShaderiv(fragmentShader, gl::COMPILE_STATUS, &mut success);
                if success != gl::TRUE as GLint {
                    gl::GetShaderInfoLog(fragmentShader, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
                    println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
                }

                gl::AttachShader(shaderProgram, vertexShader);
                gl::AttachShader(shaderProgram, fragmentShader);
                gl::LinkProgram(shaderProgram);

                gl::GetProgramiv(shaderProgram, gl::LINK_STATUS, &mut success);
                if success != gl::TRUE as GLint {
                    gl::GetProgramInfoLog(shaderProgram, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
                    println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
                }
                gl::DeleteShader(vertexShader);
                gl::DeleteShader(fragmentShader);

                (vertexShader, fragmentShader)
            };
            
       
            let vaoId = {
                //type annotation is crucial since default for float literals is f64
                let vertices: [f32; 32] = [
                    // positions     // colors           // texture coords
                    1.0,  1.0, 0.0,  1.0, 0.0, 0.0,   1.0, 1.0,   // top right
                    1.0, -1.0, 0.0,  0.0, 1.0, 0.0,   1.0, 0.0,   // bottom right
                    -1.0, -1.0, 0.0,  0.0, 0.0, 1.0,   0.0, 0.0,   // bottom let
                    -1.0,  1.0, 0.0,  1.0, 1.0, 0.0,   0.0, 1.0    // top let
                ];

                let indices = [
                    0, 1, 3,  // first Triangle
                    1, 2, 3   // second Triangle
                ];

                let (mut VBO, mut VAO, mut EBO) = (0, 0, 0);
                gl::GenVertexArrays(1, &mut VAO);
                gl::GenBuffers(1, &mut VBO);
                gl::GenBuffers(1, &mut EBO);

                gl::BindVertexArray(VAO);

                gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
                gl::BufferData(gl::ARRAY_BUFFER,
                    (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,

                    &vertices[0] as *const f32 as *const c_void,
                    gl::STATIC_DRAW);

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,

                    (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    &indices[0] as *const i32 as *const c_void,

                    gl::STATIC_DRAW);

                let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
                // position attribute
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
                gl::EnableVertexAttribArray(0);

                // color attribute
                gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
                gl::EnableVertexAttribArray(1);

                // texture coord attribute
                gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const c_void);
                gl::EnableVertexAttribArray(2);
                VAO
            };

            let texture = {
                let mut texture = 0;
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);

                texture
            };

            (shaderProgram, vaoId, texture)
        }; 

        return GLContext { glfw, window, events, shader_program_id: shaderProgram, vao: VAO, texture, texture_initilized: false};
    }

    pub fn run(&mut self) {
        while !self.gl_context.window.should_close() {
            self.process_events();

            unsafe {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                gl::BindTexture(gl::TEXTURE_2D, self.gl_context.texture);


                gl::UseProgram(self.gl_context.shader_program_id);
                gl::BindVertexArray(self.gl_context.vao); // seeing as we only have a single VAO there's no need to bind it every time, but we'll do so to keep things a bit more organized
                                          // gl::DrawArrays(gl::TRIANGLES, 0, 3);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            }

            self.gl_context.window.swap_buffers();
            self.gl_context.glfw.poll_events();
        }
    }
    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.gl_context.events) {

            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // make sure the viewport matches the new window dimensions; note that width and
                    // height will be significantly larger than specified on retina displays.
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => self.gl_context.window.set_should_close(true),

                _ => {}
            }

        }
    }
    fn update_texture(&mut self, frame: ScalerOutput) {
        unsafe {
            let data = std::slice::from_raw_parts(frame.data_ptrs[0], frame.data_linesizes[0] as usize);

            gl::BindTexture(gl::TEXTURE_2D, self.gl_context.texture); 
            gl::TexImage2D(gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const c_void
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }
}

impl Consumer<ScalerOutput> for GLRenderer {
    fn consume(&mut self, to_consume: ScalerOutput) {
        self.update_texture(to_consume);
    }
}
