//! Reflection utilities to make the OpenGL C api a bit more Rusty

use core::ffi::CStr;
use core::num::NonZero;
use core::ptr::null_mut;
use core::str::FromStr;
use std::ffi::CString;

use crate::error::checkerr;
use crate::opengl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use crate::opengl::{self, Gl};

pub(super) fn get_shader_compile_err(gl: &Gl, shader: NonZero<GLuint>) -> String {
    let mut buflen: GLint = 0;

    unsafe {
        gl.GetShaderiv(shader.get(), opengl::INFO_LOG_LENGTH, &raw mut buflen);
        checkerr!(gl);
    }

    assert!(buflen >= 0);

    let mut buf = vec![0u8; buflen as usize];

    unsafe {
        gl.GetShaderInfoLog(
            shader.get(),
            buflen,
            null_mut(),
            buf.as_mut_ptr() as *mut GLchar,
        );
        checkerr!(gl);
    }

    CStr::from_bytes_with_nul(&buf)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub(super) fn get_program_link_err(gl: &Gl, program: NonZero<GLuint>) -> String {
    let mut buflen: GLint = 0;

    unsafe {
        gl.GetProgramiv(program.get(), opengl::INFO_LOG_LENGTH, &raw mut buflen);
        checkerr!(gl);
    }

    assert!(buflen >= 0);

    let mut buf = vec![0u8; buflen as usize];

    unsafe {
        gl.GetProgramInfoLog(
            program.get(),
            buflen,
            null_mut(),
            buf.as_mut_ptr() as *mut GLchar,
        );
        checkerr!(gl);
    }

    CStr::from_bytes_with_nul(&buf)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[expect(unused)]
pub(super) fn get_active_uniforms(gl: &Gl, program: NonZero<GLuint>) -> GLuint {
    let mut num_uniforms = 0;

    unsafe {
        gl.GetProgramiv(
            program.get(),
            opengl::ACTIVE_UNIFORMS,
            &raw mut num_uniforms,
        )
    };

    checkerr!(gl);
    assert!(num_uniforms >= 0);

    num_uniforms as u32
}

#[derive(Debug, Clone)]
#[expect(unused)]
pub(super) struct ActiveUniformInfo {
    pub(super) name: String,
    pub(super) size: usize,
    pub(super) ty: GLenum,
}

#[expect(unused)]
pub(super) fn get_active_uniform(
    gl: &Gl,
    program: NonZero<GLuint>,
    uniform: GLuint,
) -> ActiveUniformInfo {
    const BUFSIZE: usize = 1024;
    let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

    let mut actual_len = 0;
    let mut size = 0;
    let mut ty = 0;

    unsafe {
        gl.GetActiveUniform(
            program.get(),
            uniform,
            BUFSIZE as GLsizei,
            &raw mut actual_len,
            &raw mut size,
            &raw mut ty,
            buf.as_mut_ptr() as *mut GLchar,
        );
    }
    checkerr!(gl);

    assert!((actual_len as usize) < (BUFSIZE - 1));

    let name_c = CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap();

    ActiveUniformInfo {
        name: name_c.to_string(),
        size: size as usize,
        ty,
    }
}

#[expect(unused)]
pub(super) fn get_active_uniform_blocks(gl: &Gl, program: NonZero<GLuint>) -> GLuint {
    let mut num_blocks = 0;

    unsafe {
        gl.GetProgramiv(
            program.get(),
            opengl::ACTIVE_UNIFORM_BLOCKS,
            &raw mut num_blocks,
        )
    };

    checkerr!(gl);
    assert!(num_blocks >= 0);

    num_blocks as u32
}

#[expect(unused)]
pub(super) fn get_active_uniform_block_uniforms(
    gl: &Gl,
    program: NonZero<GLuint>,
    uniform_block: GLuint,
) -> usize {
    let mut out = 0;

    unsafe {
        gl.GetActiveUniformBlockiv(
            program.get(),
            uniform_block,
            opengl::UNIFORM_BLOCK_ACTIVE_UNIFORMS,
            &raw mut out,
        );
    }
    checkerr!(gl);

    assert!(out >= 0);

    out as usize
}

#[expect(unused)]
pub(super) fn get_active_uniform_block_name(
    gl: &Gl,
    program: NonZero<GLuint>,
    uniform_block: GLuint,
) -> String {
    const BUFSIZE: usize = 1024;
    let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

    let mut actual_len = 0;
    unsafe {
        gl.GetActiveUniformBlockName(
            program.get(),
            uniform_block,
            BUFSIZE as GLsizei,
            &raw mut actual_len,
            buf.as_mut_ptr() as *mut GLchar,
        );
    }
    checkerr!(gl);

    assert!((actual_len as usize) < (BUFSIZE - 1));

    let name_c = CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap();

    name_c.to_string()
}

#[expect(unused)]
pub(super) fn get_active_uniform_block_iv(
    gl: &Gl,
    program: NonZero<GLuint>,
    uniform_block: GLuint,
    param: GLenum,
) -> GLint {
    assert_ne!(
        opengl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
        param,
        "Cannot query UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES. Use the dedicated function for that instead"
    );

    let mut out = 0;

    unsafe {
        gl.GetActiveUniformBlockiv(program.get(), uniform_block, param, &raw mut out);
    }

    checkerr!(gl);

    out
}

#[expect(unused)]
pub(super) unsafe fn get_active_uniform_block_indices(
    gl: &Gl,
    program: NonZero<GLuint>,
    uniform_block: GLuint,
    buf: &mut [GLuint],
) {
    unsafe {
        gl.GetActiveUniformBlockiv(
            program.get(),
            uniform_block,
            opengl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
            buf.as_mut_ptr() as *mut GLint,
        );
    }

    checkerr!(gl);
}

#[expect(unused)]
pub(super) fn get_active_uniforms_iv(
    gl: &Gl,
    program: NonZero<GLuint>,
    uniforms: &[GLuint],
    param: GLenum,
) -> Vec<GLint> {
    if uniforms.is_empty() {
        return Vec::new();
    }

    let mut to_ret = vec![0; uniforms.len()];

    unsafe {
        gl.GetActiveUniformsiv(
            program.get(),
            uniforms.len() as GLint,
            uniforms.as_ptr(),
            param,
            to_ret.as_mut_ptr(),
        );
    }

    checkerr!(gl);

    to_ret
}

pub(super) fn get_uniform_location(
    gl: &Gl,
    program: NonZero<GLuint>,
    uniform: &str,
) -> Option<GLint> {
    let name_c = CString::from_str(uniform).unwrap();

    let loc = unsafe { gl.GetUniformLocation(program.get(), name_c.as_ptr() as *const GLchar) };

    checkerr!(gl);

    if loc < 0 { None } else { Some(loc) }
}

pub(super) fn get_uniform_block_index(
    gl: &Gl,
    program: NonZero<GLuint>,
    block: &str,
) -> Option<GLuint> {
    let name_c = CString::from_str(block).unwrap();

    let index = unsafe { gl.GetUniformBlockIndex(program.get(), name_c.as_ptr() as *const GLchar) };

    checkerr!(gl);

    if index != opengl::INVALID_INDEX {
        Some(index)
    } else {
        None
    }
}
