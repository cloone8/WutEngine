use std::{env, fs::File, path::Path};

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("opengl_bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 1), Profile::Core, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
