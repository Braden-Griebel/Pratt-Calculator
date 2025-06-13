pub(crate) mod interpreter;

// Standard Library Uses

// External Uses
use anyhow::Result;
use rustyline::{self, DefaultEditor, error::ReadlineError};

// Local Uses
use crate::interpreter::interpreter::Interpreter;

fn main() -> Result<()> {
    // Create the Tree-walk interpreter
    let mut line_interpreter = Interpreter::new();
    // Create the rustyline editor
    let mut rl = DefaultEditor::new()?;
    // Print the welcome:
    print!(
        "
            Welcome to Pratt Calculator!
            This calculator uses Pratt parsing to understand then input,
            and then a simple Tree-Walk interpreter to calculate the result.
            Currently, it can handle:
                + (addition)
                - (subtraction or prefix),
                * (multiplication)
                / (division)
                ^ (exponentiation)
            as well as paranenthesis, and simple variable assignment (try `myvariable=3`).  
        "
    );
    println!("Version {}", env!("CARGO_PKG_VERSION"));
    loop {
        let readline = rl.readline(">>");
        match readline {
            Ok(line) => match line_interpreter.interpret(&line) {
                Ok(output) => println!("{output}"),
                Err(err) => println!("Interpreter Error: {err}"),
            },
            Err(ReadlineError::Interrupted) => {
                println!("Quitting...");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Quitting...");
                break;
            }
            Err(err) => {
                println!("Error: {err}");
                break;
            }
        };
    }
    Ok(())
}
