//! Build script for WutEngine Editor

use std::env;

// fn get_windows_icon() -> PathBuf {
//     let icon_png = include_bytes!("assets/Icon.png");

//     let icon_png = image::load_from_memory(icon_png).expect("Failed to load icon file");

//     let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
//     let ico_file_dest = out_dir.join("icon.ico");

//     icon_png
//         .save_with_format(&ico_file_dest, image::ImageFormat::Ico)
//         .expect("Failed to convert icon to ico");

//     ico_file_dest
// }

/// Returns the git sha for the current commit of this crate
fn get_git_sha(short: bool) -> Option<String> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")?;

    let mut command = std::process::Command::new("git");

    command.arg("-C").arg(manifest_dir).arg("rev-parse");

    if short {
        command.arg("--short");
    }

    command.arg("HEAD");

    let git_output = command.output().ok()?;

    if git_output.status.success() {
        String::from_utf8(git_output.stdout)
            .ok()
            .map(|sha| sha.trim().to_string())
    } else {
        None
    }
}

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        // let ico_file = get_windows_icon();

        let mut res = winresource::WindowsResource::new();
        // res.set_icon(ico_file.to_str().expect("Icon path is not a string"));
        res.set(
            "FileDescription",
            format!("WutEngine Editor ({})", get_git_sha(true).unwrap()).as_str(),
        );
        res.compile().expect("Failed to compile Windows RES file");
    }
}
