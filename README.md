# Pratt Calculator
A simple calculator using a Pratt parser and a
tree walk interpreter.

# Explanation
Pratt parsing is a method which uses the concept
of "binding power" to fold expressions from
the left, ultimately generating an abstract
syntax tree. It is able to handle unary, binary,
prefix, postfix expressions and paranthesis
(or other groupings). [Here](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html)
is a great blog post about implementing a Pratt parser.
If you're interested in more about parsing/programming
languages I recommend [Crafting Interpreters](https://craftinginterpreters.com/)
by Robert Nystrom. 


# Usage
Pratt-Calculator can be compiled and run using cargo,
see [here](https://www.rust-lang.org/tools/install) for
how to install rust/cargo. Once installed, you can
clone this repository:

```{shell}
  # Using ssh: 
  git clone git@github.com:Braden-Griebel/Pratt-Calculator.git
  # Or using https:
  git clone https://github.com/Braden-Griebel/Pratt-Calculator.git
```

Then you can run the binary with cargo run
```{shell}
  # Move into the clone repository
  cd Pratt-Calculator
  # Run the binary with cargo
  cargo run 
```

or compile a executable with
```{shell}
  # Build the executable
  cargo build --release
```

The executable will then be in the `target/release` folder (or directory),
and can be run as
```{shell}
  ./pratt_calculator
```
