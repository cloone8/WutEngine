//! Build script for 3D physics

fn main() {
    println!("cargo::rustc-check-cfg=cfg(phys2d, phys3d)");
    println!("cargo::rustc-cfg=phys3d");
}
