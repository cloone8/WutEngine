//! Build script for 2D physics

fn main() {
    println!("cargo::rustc-check-cfg=cfg(phys2d, phys3d)");
    println!("cargo::rustc-cfg=phys2d");
}
