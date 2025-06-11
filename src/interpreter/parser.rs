// Standard Library Uses
use std::{fmt, num, sync::atomic};

// External Crate Uses
use anyhow::{Context, Result, anyhow};

// Local Uses
use super::lexer::{AtomType, Lexer, Token};

/// An S-expression
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
enum SExprAtom {
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
        todo!()
    }
}

// Operator Binding Powers

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
