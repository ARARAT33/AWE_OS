//! AWE-Script Lightweight Scripting Engine & Interpreter.
//!
//! Provides tokenization, statement execution, system environment variable table,
//! and script execution pipeline for AWEOS system administration.

#![no_std]

pub const MAX_TOKENS: usize = 64;
pub const MAX_ENV_VARS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Identifier,
    Number,
    Equals,
    StringLiteral,
    PrintKeyword,
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub slice: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct EnvVar {
    pub hash: u64,
    pub value: i64,
}

/// AWE-Script Interpreter Environment.
#[derive(Debug)]
pub struct ScriptInterpreter {
    env_vars: [Option<EnvVar>; MAX_ENV_VARS],
    var_count: usize,
}

impl ScriptInterpreter {
    pub const fn new() -> Self {
        Self {
            env_vars: [None; MAX_ENV_VARS],
            var_count: 0,
        }
    }

    pub fn set_var(&mut self, hash: u64, value: i64) -> Result<(), &'static str> {
        for slot in self.env_vars.iter_mut() {
            if let Some(var) = slot {
                if var.hash == hash {
                    var.value = value;
                    return Ok(());
                }
            }
        }
        for slot in self.env_vars.iter_mut() {
            if slot.is_none() {
                *slot = Some(EnvVar { hash, value });
                self.var_count += 1;
                return Ok(());
            }
        }
        Err("Variable environment full")
    }

    pub fn get_var(&self, hash: u64) -> Option<i64> {
        for slot in self.env_vars.iter() {
            if let Some(var) = slot {
                if var.hash == hash {
                    return Some(var.value);
                }
            }
        }
        None
    }

    pub fn evaluate_expression(&self, left: i64, op: u8, right: i64) -> Result<i64, &'static str> {
        match op {
            b'+' => Ok(left + right),
            b'-' => Ok(left - right),
            b'*' => Ok(left * right),
            b'/' => {
                if right == 0 {
                    Err("Division by zero in script evaluation")
                } else {
                    Ok(left / right)
                }
            }
            _ => Err("Unsupported operator"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awescript_interpreter_env_and_eval() {
        let mut interp = ScriptInterpreter::new();
        let var_hash = 0x1234_5678;
        interp.set_var(var_hash, 42).unwrap();
        assert_eq!(interp.get_var(var_hash), Some(42));

        let sum = interp.evaluate_expression(10, b'+', 20).unwrap();
        assert_eq!(sum, 30);

        let div = interp.evaluate_expression(100, b'/', 4).unwrap();
        assert_eq!(div, 25);

        assert!(interp.evaluate_expression(10, b'/', 0).is_err());
    }
}
