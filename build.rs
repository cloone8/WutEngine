#[cfg(feature = "opengl")]
extern crate gl_generator;
#[cfg(feature = "opengl")]
use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

#[allow(unused_imports)]
use std::env;

#[allow(unused_imports)]
use std::fs::File;

#[allow(unused_imports)]
use std::path::Path;

#[allow(unused_imports)]
use cfg_if::cfg_if;

#[cfg(feature = "opengl")]
fn build_gl_bindings() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("opengl_bindings.rs")).unwrap();

    Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}

#[cfg(feature = "opengl")]
fn check_opengl() {
    // OpenGL is always available
}

#[cfg(feature = "vulkan")]
fn check_vulkan() {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            compile_error!("Vulkan is not available on macOS.");
        }
    }
}

#[cfg(feature = "metal")]
fn check_metal() {
    cfg_if! {
        if #[cfg(not(target_os = "macos"))] {
            compile_error!("Metal is only available on macOS.");
        }
    }
}

#[cfg(feature = "dx11")]
fn check_dx11() {
    cfg_if! {
        if #[cfg(not(target_os = "windows"))] {
            compile_error!("DirectX 11 is only available on Windows.");
        }
    }
}

#[cfg(feature = "dx12")]
fn check_dx12() {
    cfg_if! {
        if #[cfg(not(target_os = "windows"))] {
            compile_error!("DirectX 12 is only available on Windows.");
        }
    }
}

fn check_backends() {
    #[cfg(not(any(feature = "opengl", feature = "vulkan", feature = "metal", feature = "dx11", feature = "dx12")))]
    println!("cargo:warning=No rendering pipeline backend features enabled. WutEngine will not be able to render anything.");

    #[cfg(feature = "opengl")]
    check_opengl();

    #[cfg(feature = "vulkan")]
    check_vulkan();

    #[cfg(feature = "metal")]
    check_metal();

    #[cfg(feature = "dx11")]
    check_dx11();

    #[cfg(feature = "dx12")]
    check_dx12();
}

fn main() {
    check_backends();

    cfg_if! {
        if #[cfg(feature = "opengl")] {
            build_gl_bindings();
        }
    }
}
