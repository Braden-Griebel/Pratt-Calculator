// Standard Library Uses
use std::fmt;
use std::mem::take;

// External Crate Uses
use anyhow::{Context, Result, anyhow};

// Local Crate Uses

/// A single token being parsed
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    Op(char),
    Atom(AtomType),
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Op(c) => write!(f, "{}", c),
            Token::Atom(at) => match at {
                AtomType::Number(n) => write!(f, "{}", n),
                AtomType::Variable(varname) => write!(f, "{}", varname),
            },
            Token::EOF => write!(f, "EOF"),
        }
    }
}

impl Token {
    /// Create a new Token representing an operation
    fn new_op(operator: char) -> Result<Self> {
        Ok(Self::Op(operator))
    }

    /// Create a new Token representing a number
    fn new_number(num: &str) -> Result<Self> {
        Ok(Token::Atom(AtomType::new_num(num)?))
    }

    /// Create a new Token representing a variable
    fn new_variable(var_name: &str) -> Result<Self> {
        Ok(Token::Atom(AtomType::new_variable(var_name)?))
    }
}

/// The possible types of an Atom
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AtomType {
    /// A single floating point number
    Number(f64),
    /// A variable identifier
    Variable(String),
}

impl AtomType {
    /// Create a new number Atom
    fn new_num(num: &str) -> Result<Self> {
        let internal_num = num.parse::<f64>().context("Failed to parse number")?;
        Ok(AtomType::Number(internal_num))
    }

    /// Create a new variable Atom
    fn new_variable(var_name: &str) -> Result<Self> {
        Ok(AtomType::Variable(var_name.to_string()))
    }
}

/// Lexes a string into a sequence of Tokens
pub(crate) struct Lexer {
    /// The generated sequence of tokens
    tokens: Vec<Token>,
    /// The input being Lexed
    input: Vec<char>,
    /// The current position in the input
    current_position: usize,
    /// The start position of the current token being lexed
    start_position: usize,
}

// Create Lexer
impl Lexer {
    /// Create a new lexer
    pub(crate) fn new(input: &str) -> Result<Self> {
        let input_vec = input.trim().to_string().chars().collect::<Vec<char>>();
        Ok(Self {
            tokens: Vec::new(),
            input: input_vec,
            current_position: 0usize,
            start_position: 0usize,
        })
    }
}

// Main lexer functions
impl Lexer {
    /// Lex the input into a series of Tokens
    pub(crate) fn lex(&mut self) -> Result<Vec<Token>> {
        while !self.at_end() {
            self.start_position = self.current_position;
            let cur_char = self
                .pop()
                .context("Failed to get next character during lexing")?;
            match cur_char {
                // Match all the operators
                '(' | ')' | '*' | '/' | '+' | '-' | '^' | '!' | '=' => self.tokens.push(
                    Token::new_op(cur_char)
                        .context("Unable to create new operator token during lexing")?,
                ),
                // Match possible starts of variable names
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.consume_variable()?;
                    let new_var_name =
                        match self.input.get(self.start_position..self.current_position) {
                            Some(s) => s.to_vec().iter().collect::<String>(),
                            None => {
                                return Err(anyhow!(
                                    "Ater consuming variable, unable to get the variables name"
                                ));
                            }
                        };
                    self.tokens.push(
                        Token::new_variable(&new_var_name)
                            .context("Unable to create new variable from consumed variable")?,
                    );
                }
                // Match the start of a number
                '0'..='9' => {
                    self.consume_number()?;
                    let new_num: String =
                        match self.input.get(self.start_position..self.current_position) {
                            Some(s) => s.iter().collect::<String>(),
                            None => {
                                return Err(anyhow!(
                                    "After consuming number, unable to retrieve the number"
                                ));
                            }
                        };
                    self.tokens.push(
                        Token::new_number(&new_num)
                            .context("Unable to create new number token from consumed number")?,
                    );
                }
                // Match spaces (and other whitespace)
                c if c.is_whitespace() => {}
                // Any other characters are unexpected, return Err
                _ => {
                    return Err(anyhow!(
                        "Unexpected character encountered during lexing: {cur_char}"
                    ));
                }
            }
        }

        // Now that lexing has reached the end, append an EOF token, and return the sequence
        self.tokens.push(Token::EOF);
        Ok(take(&mut self.tokens))
    }

    /// Increment current position until it is past the end of the variable
    fn consume_variable(&mut self) -> Result<()> {
        while !self.at_end() && self.is_valid_var().context("Failed to consume variable")? {
            self.consume();
        }

        Ok(())
    }

    /// Increment current position until it is past the end of a number
    fn consume_number(&mut self) -> Result<()> {
        let mut encounted_decimal = false;

        while !self.at_end() {
            let cur_char = self.peek()?;
            match cur_char {
                '0'..='9' => {
                    self.consume();
                }
                '.' => {
                    if encounted_decimal {
                        return Err(anyhow!(
                            "Encountered two decimal points in single number during lexing"
                        ));
                    }
                    encounted_decimal = true;
                    self.consume();
                }
                _ => {
                    break;
                }
            }
        }

        Ok(())
    }
}

// Some utility methods for the lexer
impl Lexer {
    /// Return the next character without consuming it
    fn peek(&self) -> Result<char> {
        if let Some(c) = self.input.get(self.current_position) {
            return Ok(c.clone());
        }
        Err(anyhow!("Tried to index past end of input during lexing"))
    }

    /// Consume the next character and return it
    fn pop(&mut self) -> Result<char> {
        let next_char = self.peek()?;
        self.current_position += 1;
        Ok(next_char)
    }

    /// Consume the next character, not returning it
    fn consume(&mut self) -> () {
        self.current_position += 1;
    }

    /// Determine if entire input has been parsed
    fn at_end(&self) -> bool {
        self.current_position >= self.input.len()
    }

    // /// Check whether the current character is a letter
    // fn is_alpha(&self) -> Result<bool> {
    //     Ok((self
    //         .input
    //         .get(self.current_position)
    //         .ok_or(anyhow!("Lexer continued parsing past end of the input")))?
    //     .is_alphabetic())
    // }

    // /// Check whether the current character is a number
    // fn is_numeric(&self) -> Result<bool> {
    //     Ok((self
    //         .input
    //         .get(self.current_position)
    //         .ok_or(anyhow!("Lexer continued past end of the input")))?
    //     .is_numeric())
    // }

    // /// Check whether the current character is alphanumeric
    // fn is_alphanumeric(&self) -> Result<bool> {
    //     Ok(self.is_alpha()? && self.is_numeric()?)
    // }

    /// Check whether the current character is a valid variable character
    fn is_valid_var(&self) -> Result<bool> {
        let cur_char = self
            .peek()
            .context("Lexer continued past end of input while parsing a variable")?;
        Ok(cur_char.is_alphanumeric() || cur_char == '_')
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_lex_number() -> Result<()> {
        // Create the test lexer
        let mut test_lexer = Lexer::new("3.14")?;
        // Run the lexer
        let lexed_tokens = test_lexer.lex()?;
        // Test that the token created is correct
        let test_token = match lexed_tokens.get(0) {
            Some(t) => t,
            None => {
                return Err(anyhow!("Lexing returned an empty vector"));
            }
        };
        // Check that the lexed token is correct
        match test_token {
            Token::Atom(atom_type) => match atom_type {
                AtomType::Number(n) => {
                    if (n - 3.14f64) > 0.0000001f64 {
                        return Err(anyhow!("Lexer returned incorrect value of number"));
                    }
                }
                _ => return Err(anyhow!("Lexing returned incorrect AtomType")),
            },
            _ => return Err(anyhow!("Lexing returned incorrect token type")),
        }
        Ok(())
    }

    #[test]
    fn test_lex_variable() -> Result<()> {
        // Create the test lexer
        let mut test_lexer = Lexer::new("myvariable")?;
        // Run the lexer
        let lexed_tokens = test_lexer.lex()?;
        // Test that the token created is correct
        let test_token = match lexed_tokens.get(0) {
            Some(t) => t,
            None => {
                return Err(anyhow!("Lexing returned an empty vector"));
            }
        };
        // Check that the lexed token is correct
        match test_token {
            Token::Atom(atom_type) => match atom_type {
                AtomType::Variable(varname) => {
                    assert_eq!(varname, "myvariable")
                }
                _ => return Err(anyhow!("Lexing returned incorrect AtomType")),
            },
            _ => return Err(anyhow!("Lexing returned incorrect token type")),
        }
        Ok(())
    }

    #[test]
    fn test_lex_op() -> Result<()> {
        // Create the test lexer
        let mut test_lexer = Lexer::new("+")?;
        // Run the lexer
        let lexed_tokens = test_lexer.lex()?;
        // Test that the token created is correct
        let test_token = match lexed_tokens.get(0) {
            Some(t) => t,
            None => {
                return Err(anyhow!("Lexing returned an empty vector"));
            }
        };

        match test_token {
            Token::Op(operator) => {
                assert_eq!(operator, &'+');
            }
            _ => return Err(anyhow!("Lexer returned incorrect token type")),
        }

        Ok(())
    }

    #[test]
    fn test_lex_series() -> Result<()> {
        // Create the test lexer
        let mut test_lexer = Lexer::new("(3.14)* 5+a/ myvariable")?;
        // Run the lexer
        let lexed_tokens = test_lexer.lex()?;
        // Create a vec of the expected output
        let expected_tokens: Vec<Token> = vec![
            Token::Op('('),
            Token::Atom(AtomType::Number(3.14)),
            Token::Op(')'),
            Token::Op('*'),
            Token::Atom(AtomType::Number(5f64)),
            Token::Op('+'),
            Token::Atom(AtomType::Variable("a".to_string())),
            Token::Op('/'),
            Token::Atom(AtomType::Variable("myvariable".to_string())),
            Token::EOF,
        ];
        // Check that the lexed output is as expected
        assert_eq!(lexed_tokens, expected_tokens);
        Ok(())
    }
}
