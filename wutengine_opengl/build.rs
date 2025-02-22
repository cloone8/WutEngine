use std::{env, fs::File, io::Write, path::Path};

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

/// Generate OpenGL bindings using the given configuration.
/// 
/// # Parameters
/// - `api`: The graphics API (e.g. Api::Gl or Api::Gles).
/// - `version`: The version tuple (major, minor).
/// - `profile`: The profile to use (Core or Compatibility).
/// - `fallbacks`: Fallback strategy for missing symbols.
/// - `extensions`: A slice of additional extension names.
/// 
/// # Returns
/// Returns Ok(()) on success or an error if binding generation fails.
fn gen_gl_bindings(
    api: Api,
    version: (u8, u8),
    profile: Profile,
    fallbacks: Fallbacks,
    extensions: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("gl_generated_bindings.rs");
    let mut file = File::create(&dest_path)?;
    
    Registry::new(api, version, profile, fallbacks, extensions)
        .write_bindings(StructGenerator, &mut file)?;
    
    println!("Generated GL bindings at: {:?}", dest_path);
    Ok(())
}

fn main() {
    // Read configuration from environment variables, with defaults.
    // Set GL_API to "gles" for OpenGL ES, otherwise defaults to regular OpenGL.
    let api = match env::var("GL_API").as_deref() {
        Ok("gles") => Api::Gles,
        _ => Api::Gl,
    };

    // Read the GL version from environment variable (default "3.3")
    let version_str = env::var("GL_VERSION").unwrap_or_else(|_| "3.3".to_string());
    let version_parts: Vec<&str> = version_str.split('.').collect();
    let major = version_parts.get(0).and_then(|s| s.parse::<u8>().ok()).unwrap_or(3);
    let minor = version_parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(3);

    // Read GL_PROFILE from env ("compatibility" for Compatibility, default Core)
    let profile = match env::var("GL_PROFILE").as_deref() {
        Ok("compatibility") => Profile::Compatibility,
        _ => Profile::Core,
    };

    // Use Fallbacks::All by default (could be made configurable if needed)
    let fallbacks = Fallbacks::All;

    // Read extensions as a comma-separated list (optional)
    let extensions: Vec<&str> = env::var("GL_EXTENSIONS")
        .ok()
        .map(|ext| {
            ext.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_else(Vec::new);

    // Generate the bindings, exiting with an error if it fails.
    if let Err(e) = gen_gl_bindings(api, (major, minor), profile, fallbacks, &extensions) {
        eprintln!("
