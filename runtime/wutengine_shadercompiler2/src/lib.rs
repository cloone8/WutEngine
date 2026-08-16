#![doc = include_str!("../README.md")]

use pest::Parser;
use pest_derive::Parser;

#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum CompileErr {
    #[display("Failed to parse WGSL: {_0}")]
    CompileIR(Box<naga::front::wgsl::ParseError>),
}

#[derive(Debug)]
pub struct CompileOutput {
    pub compiled_module: naga::Module,
}

pub fn compile(input: &str) -> Result<Box<CompileOutput>, CompileErr> {
    log::info!("Compiling input");

    // preproc
    let input = preprocess(input);
    // compile

    let mut naga_frontend =
        naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
            parse_doc_comments: true,
            capabilities: naga::valid::Capabilities::default(),
        });

    let module = naga_frontend.parse(&input).map_err(Box::new)?;

    // dbg!(&module);

    // for entry_point in &module.entry_points {
    //     log::info!("Entrypoint: {} ({:?})", entry_point.name, entry_point.stage);
    //     dump_func(&module, &entry_point.function);
    // }

    // for (_, function) in module.functions.iter() {
    //     dump_func(&module, function);
    // }

    // for (_, global_var) in module.global_variables.iter() {
    //     log::info!("Global Variable {:?}", global_var.name);

    //     log::info!("{:#?}", global_var.binding);
    //     log::info!("{:#?}", global_var.space);
    //     log::info!("{:#?}", module.types[global_var.ty]);
    // }

    Ok(Box::new(CompileOutput {
        compiled_module: module,
    }))
}

fn dump_func(module: &naga::ir::Module, f: &naga::ir::Function) {
    log::info!("Function: {:?}", f.name);

    for arg in &f.arguments {
        log::info!(
            "Arg {:#?}: {:#?} {:?}",
            arg.name,
            module.types[arg.ty],
            arg.binding
        );
    }

    log::info!("Return: {:#?}", f.result);
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct WutEngineShaderParser;

fn preprocess(input: &str) -> String {
    pest::set_error_detail(true);

    let a = match WutEngineShaderParser::parse(Rule::shader_file, input) {
        Ok(a) => a,
        Err(e) => {
            log::error!("{e}");
            panic!();
        }
    };

    dbg!(a);

    input.to_string()
}
