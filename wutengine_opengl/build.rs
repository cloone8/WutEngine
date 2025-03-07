//! Build script for the WutEngine OpenGL backend. Generates the proper bindings

use std::{env, fs::File, path::Path};

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

/// Generate the bindings
fn gen_gl_bindings() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut gl_binding_dst =
        File::create(Path::new(&out_dir).join("gl_generated_bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 1), Profile::Core, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut gl_binding_dst)
        .unwrap();
}

fn main() {
    gen_gl_bindings();
}
