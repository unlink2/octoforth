extern crate clap;
extern crate octoforth;
use clap::{AppSettings, Clap};
use octoforth::compiler::Compiler;
use octoforth::error::BoxResult;
use octoforth::stmt::Compiled;

#[derive(Clap)]
#[clap(version = "1.0", author = "Lukas Krickl <lukas@krickl.dev>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long)]
    input: String
}

fn main() -> BoxResult<()> {
    let opts: Opts = Opts::parse();

    let mut compiler = Compiler::from_file(&opts.input)?;

    let result = compiler.compile()?;
    println!("{}", Compiled::flatten(result)?);

    return Ok(());
}
