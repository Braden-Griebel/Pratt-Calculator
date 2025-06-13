//! Implementation of a Tree-Walk interpreter
// Standard Library Uses
use std::collections::HashMap;

// External Uses
use anyhow::{Context, Result, anyhow};

// Local Uses
use super::parser::{PrattParser, SExpr, SExprAtom};

/// A Tree Walk interpreter
struct Interpreter {
    environment: HashMap<String, f64>,
}

impl Interpreter {
    /// Create a new interpreter with an empty environment
    fn new() -> Self {
        Interpreter {
            environment: HashMap::new(),
        }
    }

    /// Interpret a program represented as a string
    fn interpret(&mut self, input: &str) -> Result<f64> {
        let program_sexpr = PrattParser::parse(input)
            .context("Trying to parse input into S-expression for interpretation")?;
        self.interpret_sexpr(program_sexpr)
    }

    /// Interpret an S-expression, returning a numerical value, or an error
    fn interpret_sexpr(&mut self, expr: SExpr) -> Result<f64> {
        match expr {
            SExpr::Atom(at) => match at {
                SExprAtom::Op(_) => Err(anyhow!(
                    "Encountered operator as S-expression atom with no operands"
                )),
                SExprAtom::Number(num) => Ok(num),
                SExprAtom::Variable(varname) => match self.environment.get(&varname) {
                    Some(val) => Ok(val.to_owned()),
                    None => Err(anyhow!("Tried to access variable with no value assigned")),
                },
            },
            SExpr::Cons(operator, mut operands) => match operator {
                SExprAtom::Op(op) => match op {
                    // Match prefix operators
                    '+' | '-' if operands.len() == 1 => {
                        let operand_value = match operands.pop() {
                            Some(val) => val,
                            None => {
                                return Err(anyhow!(
                                    "Failed to extract value from prefix + operand"
                                ));
                            }
                        };
                        Ok(self.interpret_sexpr(operand_value)?
                            * (if op == '+' {
                                1f64 // Prefix + is a no-op
                            } else if op == '-' {
                                -1f64 // Multiply by -1
                            } else {
                                // This should never happen
                                return Err(anyhow!(
                                    "Inavlid operator, matched a + or - but is neither"
                                ));
                            }))
                    }
                    // Match Binary Operators (excluding assignment)
                    '+' | '-' | '*' | '/' | '^' if operands.len() == 2 => {
                        // Extract the operands
                        let rhs = match operands.pop() {
                            Some(val) => val,
                            None => {
                                return Err(anyhow!(
                                    "
                                        Unable to extract right hand side of binary operator"
                                ));
                            }
                        };
                        let lhs = match operands.pop() {
                            Some(val) => val,
                            None => {
                                return Err(anyhow!(
                                    "Unable to extract left hand side of binary operator"
                                ));
                            }
                        };
                        // Evaluate the operands
                        let lhs_value = self
                            .interpret_sexpr(lhs)
                            .context("Failed to evaluate lhs of binary operator")?;
                        let rhs_value = self
                            .interpret_sexpr(rhs)
                            .context("Failed to evaluate rhs of binary operator")?;

                        // Now compute the result
                        let res = match op {
                            '+' => lhs_value + rhs_value,
                            '-' => lhs_value - rhs_value,
                            '*' => lhs_value * rhs_value,
                            '/' => lhs_value / rhs_value,
                            '^' => lhs_value.powf(rhs_value),
                            _ => return Err(anyhow!("Encountered invalid binary operator {op}")),
                        };

                        // Return the result of the computation
                        Ok(res)
                    }
                    // Match the assignment operator
                    '=' if operands.len() == 2 => {
                        let rhs = match operands.pop() {
                            Some(sexpr) => self
                                .interpret_sexpr(sexpr)
                                .context("Unable to evaluate rhs of assignment")?,
                            None => return Err(anyhow!("Assignment operator had no operands")),
                        };
                        match operands.pop() {
                            Some(sexpr) => match sexpr {
                                SExpr::Atom(at) => match at {
                                    SExprAtom::Variable(varname) => {
                                        self.environment.insert(varname, rhs);
                                        Ok(rhs)
                                    }
                                    _ => Err(anyhow!(
                                        "Invalid lhs of assignment operator encountered: {at}"
                                    )),
                                },
                                _ => Err(anyhow!(
                                    "Invalid lhs of assignment operator encountered: {sexpr}"
                                )),
                            },
                            None => Err(anyhow!("No lhs of assignment operator")),
                        }
                    }
                    // Finally the postfix operators
                    '!' if operands.len() == 1 => {
                        let lhs = match operands.pop() {
                            Some(val) => self.interpret_sexpr(val)?,
                            None => {
                                return Err(anyhow!("Unable to extranct operand for factorial"));
                            }
                        } as i32;
                        let mut res = 1;
                        let mut iterator = lhs.abs();
                        while iterator > 0 {
                            res *= iterator;
                            iterator -= 1;
                        }
                        if lhs < 0 {
                            res *= -1;
                        }
                        Ok(res as f64)
                    }
                    _ => Err(anyhow!(
                        "Encountered invalid S-expresion ({operator} {operands:?})"
                    )),
                },
                _ => Err(anyhow!(
                    "Encountered a variable or number ({operator}) as operator in S-expression"
                )),
            },
        }
    }
}

#[cfg(test)]
mod test_interpreter {
    use super::*;

    #[test]
    fn test_atom() -> Result<()> {
        let mut test_interpreter = Interpreter::new();
        assert_eq!(test_interpreter.interpret("3")?, 3f64);
        Ok(())
    }

    #[test]
    fn test_binary_operator() -> Result<()> {
        let mut test_interpreter = Interpreter::new();
        assert_eq!(test_interpreter.interpret("3+4")?, 7f64);
        assert_eq!(test_interpreter.interpret("3*4")?, 12f64);
        assert_eq!(test_interpreter.interpret("2^3")?, 8f64);
        Ok(())
    }

    #[test]
    fn test_postfix_operator() -> Result<()> {
        let mut test_interpreter = Interpreter::new();
        assert_eq!(test_interpreter.interpret("3!")?, 6f64);
        Ok(())
    }

    #[test]
    fn test_variable_assignment() -> Result<()> {
        let mut test_interpreter = Interpreter::new();
        assert_eq!(test_interpreter.interpret("a=3")?, 3f64);
        assert_eq!(test_interpreter.interpret("a+4")?, 7f64);
        Ok(())
    }
}
