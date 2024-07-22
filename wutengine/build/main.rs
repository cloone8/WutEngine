use std::{env, fs::File, io::Write, path::PathBuf};

mod componentfilter;

fn gen_componentfilters() {
    let componentfilter_impls = componentfilter::make_componentfilter_tuples(5);

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("componentfilters.rs");

    let mut out_file = File::create(out_path).unwrap();
    out_file
        .write_all(componentfilter_impls.to_string().as_bytes())
        .unwrap();
}

fn main() {
    gen_componentfilters();
}
