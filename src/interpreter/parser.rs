// Standard Library Uses
use std::fmt;

// External Crate Uses
use anyhow::{Context, Result, anyhow};

// Local Uses
use super::lexer::{AtomType, Lexer, Token};

/// An S-expression
#[derive(Clone, Debug)]
pub(crate) enum SExpr {
    Atom(SExprAtom),
    Cons(SExprAtom, Vec<SExpr>),
}

impl fmt::Display for SExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SExpr::Atom(at) => {
                write!(f, "{}", at)
            }
            SExpr::Cons(op, args) => {
                write!(f, "({}", op)?;
                for at in args {
                    write!(f, " {}", at)?
                }
                write!(f, ")")
            }
        }
    }
}

/// An S-expression atom
#[derive(Clone, Debug)]
pub(crate) enum SExprAtom {
    /// An operation such as +, -, etc.
    Op(char),
    /// A variable identifier
    Variable(String),
    /// A floating point number
    Number(f64),
}

impl fmt::Display for SExprAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SExprAtom::Op(operation) => {
                write!(f, "{}", operation)
            }
            SExprAtom::Variable(variable_name) => {
                write!(f, "{}", variable_name)
            }
            SExprAtom::Number(num) => {
                write!(f, "{}", num)
            }
        }
    }
}

/// Parses sequences of Tokens into S-expressions
pub(crate) struct PrattParser {
    /// Series of tokens to parse
    tokens: Vec<Token>,
}

// Main Parsing Functions
impl PrattParser {
    /// Parse a string into an S-expression
    pub(crate) fn parse(input: &str) -> Result<SExpr> {
        let mut parser = PrattParser::new(input)?;
        Ok(parser.parse_min_bp(0u8)?)
    }

    fn parse_min_bp(&mut self, min_bp: u8) -> Result<SExpr> {
        // "Priming the pumnp"
        // Parsing the initial characters to get things started,
        // Setting up the lhs, and the rhs will be parsed
        // through the loop below
        let mut lhs = match self
            .pop()
            .context("Tried to pop next token during parsing")?
        {
            Token::Atom(at) => match at {
                AtomType::Number(n) => SExpr::Atom(SExprAtom::Number(n)),
                AtomType::Variable(varname) => SExpr::Atom(SExprAtom::Variable(varname)),
            },
            Token::Op('(') => {
                let lhs = self.parse_min_bp(0u8)?;
                if self.pop()? != Token::Op(')') {
                    return Err(anyhow!("Unmatched paranthesis encountered during parsing"));
                }
                lhs
            }
            Token::Op(op) => {
                let ((), bp) = Self::prefix_binding_power(&op).context(
                    "Trying to determine binding power of first token encountered in Pratt Parser",
                )?;
                let rhs = self.parse_min_bp(bp)?;
                SExpr::Cons(SExprAtom::Op(op), vec![rhs])
            }
            t => return Err(anyhow!("Encountered bad token during parsing {t}")),
        };

        // Parse the rhs of the above expression
        loop {
            // Start by checking the next character, if it is an EOF Break
            // If it is an operator that will be further processed
            // Otherwise, it's a parsing error
            let op = match self
                .peek()
                .context("Peeking next token during rhs parsing loop")?
            {
                Token::EOF => break,
                Token::Op(op) => op,
                t => {
                    return Err(anyhow!(
                        "Encountered unknown token {t} during rhs parsing loop"
                    ));
                }
            };

            // Start by seeing if this operator may be a postfix operator
            if let Some((pf_bp, ())) = Self::postfix_binding_power(&op) {
                // If the postfix binding power is too low,
                // the loop should be broken as parsing has finished
                if pf_bp < min_bp {
                    break;
                }

                // Otherwise, consume the Token holding the operator
                self.consume()?;

                // Then update the lhs to add the postfix oepration
                lhs = SExpr::Cons(SExprAtom::Op(op), vec![lhs]);

                // Now that the lhs has been updated, continue to the
                // next iteration
                continue;
            }

            // If the operation is not a postfix operator,
            // process it as an infix operator
            if let Some((l_bp, r_bp)) = Self::infix_binding_power(&op) {
                // Check if the binding power is too low
                if l_bp < min_bp {
                    // Note: Since we are binding it to the left expression,
                    // only the l_bp is of interest
                    break;
                }
                // Consume the token since it is an infix operator
                self.consume()?;

                // Process the rhs
                lhs = {
                    let rhs = self.parse_min_bp(r_bp).context(
                        "Failed to parse right hand side of infix operator during parsing",
                    )?;
                    SExpr::Cons(SExprAtom::Op(op), vec![lhs, rhs])
                };

                // Now that the lhs has been updated, continue to the
                // next iteration
                continue;
            }

            // The parsing has now finished, so break the loop
            break;
        }

        Ok(lhs)
    }
}

// Operator Binding Powers
impl PrattParser {
    /// Determine the infix binding power of the operator
    /// represented by c
    fn infix_binding_power(c: &char) -> Option<(u8, u8)> {
        match c {
            '=' => Some((2, 1)),
            '+' | '-' => Some((3, 4)),
            '^' => Some((6, 5)),
            '*' | '/' => Some((7, 8)),
            _ => None,
        }
    }

    /// Determine the prefix binding power of the operator
    /// represented by c
    fn prefix_binding_power(c: &char) -> Result<((), u8)> {
        match c {
            '+' | '-' => Ok(((), 9)),
            _ => Err(anyhow!(
                "Character {c} does not have an associated prefix binding power"
            )),
        }
    }

    /// Determine the postfix binding power of the operator
    /// represented by c
    fn postfix_binding_power(c: &char) -> Option<(u8, ())> {
        match c {
            '!' => Some((11, ())),
            _ => None,
        }
    }
}

// Utility functions for the Parser
impl PrattParser {
    /// Create a new Parser from a string input
    fn new(input: &str) -> Result<Self> {
        // Create a parser from the input
        let mut parser_lexer = Lexer::new(input)?;
        // Lex the input into a series of tokens
        let mut tokens = parser_lexer
            .lex()
            .context("Failed to parse input to parser")?;
        // Reverse the tokens to make popping easier
        tokens.reverse();
        Ok(Self { tokens })
    }

    /// Get the next token without consuming it
    fn peek(&self) -> Result<Token> {
        Ok(self.tokens.last().cloned().unwrap_or(Token::EOF))
    }

    /// Get the next token and consume it
    fn pop(&mut self) -> Result<Token> {
        Ok(self.tokens.pop().unwrap_or(Token::EOF))
    }

    /// Consume the next token, returning nothing
    fn consume(&mut self) -> Result<()> {
        _ = self.pop();
        Ok(())
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;

    #[test]
    fn test_atom_parsing() -> Result<()> {
        let program = "3.14";
        let parsed_res = PrattParser::parse(program)?;
        match parsed_res {
            SExpr::Atom(seatom) => match seatom {
                SExprAtom::Number(num) => {
                    if num == 3.14f64 {
                        return Ok(());
                    } else {
                        return Err(anyhow!("Incorrect atom value found!"));
                    }
                }
                _ => return Err(anyhow!("Incorrect atom type found!")),
            },
            _ => return Err(anyhow!("Incorrect S-expression type found!")),
        }
    }

    #[test]
    fn test_simple_expression_parsing() -> Result<()> {
        let program = "3 + 4";
        let parsed_res = PrattParser::parse(program)?;
        let expected = "(+ 3 4)";
        assert_eq!(parsed_res.to_string(), expected);
        Ok(())
    }

    #[test]
    fn test_operator_precedence() -> Result<()> {
        let program = "3+5*6";
        let parsed_res = PrattParser::parse(program)?;
        let expected = "(+ 3 (* 5 6))";
        assert_eq!(parsed_res.to_string(), expected);
        Ok(())
    }
}
