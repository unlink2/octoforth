extern crate clap;
extern crate octoforth;
use clap::{AppSettings, Clap};
use octoforth::compiler::Compiler;
use octoforth::error::BoxResult;
use octoforth::stmt::Compiled;
use std::io::Write;

#[derive(Clap)]
#[clap(version = "1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    input: String,
    output: Option<String>
}

fn main() -> BoxResult<()> {
    let opts: Opts = Opts::parse();

    let mut compiler = match Compiler::from_file(&opts.input) {
        Ok(compiler) => compiler,
        Err(err) => {
            println!("{}", err);
            return Err(Box::new(err));
        }
    };

    let mut result = match compiler.compile() {
      Ok(result) => result,
      Err(err) => {
        println!("{}", err);
        return Err(Box::new(err));
      }
    };

    match opts.output {
        Some(s) => {
            let mut file = std::fs::File::create(s)?;
            file.write_all(&mut Compiled::flatten_bytes(&mut result).data)?;
        },
        _ => {
            println!("{}", Compiled::flatten(result)?);
        }
    }

    return Ok(());
}
