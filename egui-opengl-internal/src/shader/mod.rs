use gl::types::{GLenum, GLuint, GLint, GLchar};

pub struct Shader;

impl Shader {
    pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
        let id = unsafe { gl::CreateShader(ty) };
        unsafe {
            let ptr: *const u8 = src.as_bytes().as_ptr();
            let ptr_i8: *const i8 = std::mem::transmute(ptr);
            let len = src.len() as GLint;
            gl::ShaderSource(id, 1, &ptr_i8, &len);
        }
    
        let successful = unsafe {
            gl::CompileShader(id);
    
            let mut result: GLint = 0;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
            result != 0
        };
    
        if !successful {
            panic!()
        }
    
        id
    }
    
    pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
        let program = unsafe { gl::CreateProgram() };
    
        unsafe {
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
        }
    
        let mut status = gl::FALSE as GLint;
        unsafe {
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        }
    
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            unsafe {
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            }
    
            let mut buf = vec![0; len as usize];
    
            unsafe {
                gl::GetProgramInfoLog(
                    program,
                    len,
                    core::ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
            }
    
            panic!(
                "{}",
                core::str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
            );
        }
    
        program
    }
}
