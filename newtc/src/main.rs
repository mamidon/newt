#![allow(unused)]


#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate insta;

mod featurez;

use crate::featurez::*;

use std::env::args;
use std::io::Write;
use std::io::{stdin, stdout};
use std::path::PathBuf;
use std::str::Chars;

struct Config {
    output_mode: OutputMode,
    entry_file: Option<PathBuf>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum OutputMode {
    Tokens,
    ParseTree,
    AbstractSyntaxTree,
}

fn main() {
    let arguments: Vec<String> = args().collect();

    let borrowed_arguments = arguments.iter().map(|s| s.as_ref()).collect();

    let config = Config::parse(&borrowed_arguments);

    match config {
        Some(c) => {
            let entry_file_contents = c.entry_file.and_then(|f| std::fs::read_to_string(f).ok());

            if let Some(entry_file_contents) = entry_file_contents {
                batch(entry_file_contents, c.output_mode)
            } else {
                repl(c.output_mode)
            }
        }
        None => print_help(),
    }
}

fn batch(file: String, output_mode: OutputMode) {
    match output_mode {
        OutputMode::Tokens => token_batch(file),
        OutputMode::ParseTree => parse_batch(&file, &mut VirtualMachine::new()),
        _ => unimplemented!(
            "Have not yet implemented batch processing for {:?}",
            output_mode
        ),
    }
}

fn repl(output_mode: OutputMode) {
    match output_mode {
        OutputMode::Tokens => token_repl(),
        OutputMode::ParseTree => parse_repl(),
        _ => unimplemented!("Have not yet implemented repl for {:?}", output_mode),
    }
}

fn token_batch(file: String) {
    let tokens = tokenize(&file);

    print_tokens(&file, &tokens);
}

fn token_repl() {
    let mut input_buffer = String::new();
    loop {
        input_buffer.clear();
        print!("newt> ");
        stdout().flush().ok().expect("failed to write to stdout");

        stdin().read_line(&mut input_buffer);
        let sanitized_input = input_buffer.trim();

        if sanitized_input.len() == 0 {
            break;
        }

        let tokens = tokenize(&sanitized_input);

        print_tokens(&sanitized_input, &tokens);
    }
}

fn parse_repl() {
    let mut input_buffer = String::new();
    let mut machine = VirtualMachine::new();

    loop {
        if input_buffer.is_empty() {
            print!("newt> ");
        } else {
            print!("> ");
        }

        stdout().flush().ok().expect("failed to write to stdout");

        stdin().read_line(&mut input_buffer);
        let sanitized_input = input_buffer.trim();

        if sanitized_input.len() == 0 {
            break;
        }

        if balanced_braces(&input_buffer) {
            parse_batch(&input_buffer, &mut machine);
            input_buffer.clear();
        }
    }
}

fn balanced_braces(input_buffer: &str) -> bool {
    let mut braces_counted = 0;
    let mut parenthesis_counted = 0;

    for c in input_buffer.chars() {
        match c {
            '{' => braces_counted = braces_counted + 1,
            '}' => braces_counted = braces_counted - 1,
            '(' => parenthesis_counted = parenthesis_counted + 1,
            ')' => parenthesis_counted = parenthesis_counted - 1,
            _ => {}
        };

        if braces_counted < 0 || parenthesis_counted < 0 {
            break;
        }
    }

    return braces_counted == 0 && parenthesis_counted == 0;
}

fn parse_batch(file: &str, machine: &mut VirtualMachine) {
	let session = InterpretingSession {
        kind: InterpretingSessionKind::Stmt,
        source: file
    };

	let tree = build(session);

    println!("{}", tree);

    let result = interpret(machine, &tree);

    println!("RESULT: {:?}", result);
    println!("STATE: {:#?}", machine);
}

fn print_tokens(source_text: &str, tokens: &Vec<Token>) {
    let mut offset = 0;

    for token in tokens {
        let end = offset + token.lexeme_length();

        match token.token_kind() {
            TokenKind::TombStone => println!("{} '{}'", token, &source_text[offset..end]),
            TokenKind::WhiteSpace => {
                let mut printable_whitespace = String::new();

                for c in source_text[offset..end].chars() {
                    match c {
                        ' ' => printable_whitespace.push_str("\\s"),
                        '\t' => printable_whitespace.push_str("\\t"),
                        '\n' => printable_whitespace.push_str("\\n"),
                        _ => printable_whitespace.push(c),
                    }
                }

                println!("{} '{}'", token, printable_whitespace);
            }
            _ => println!("{} '{}'", token, &source_text[offset..end]),
        }

        offset = end;
    }

    println!();
}

fn print_help() {
    println!("usage: newtc [--entry-file (path)] --output-mode (tokens | parse-tree | ast)");
}

impl Config {
    pub fn parse(arguments: &Vec<&str>) -> Option<Config> {
        if let (Some(output_mode), entry_file) = (
            Config::parse_output_mode(arguments),
            Config::parse_entry_file(arguments),
        ) {
            Some(Config {
                output_mode,
                entry_file,
            })
        } else {
            None
        }
    }

    fn parse_entry_file(arguments: &Vec<&str>) -> Option<PathBuf> {
        let entry_file_flag_position = arguments.iter().position(|arg| *arg == "--entry-file");

        match entry_file_flag_position {
            Some(position) => {
                let entry_file = arguments
                    .get(position + 1)
                    .and_then(|s| Some(PathBuf::from(s)));
                return entry_file;
            }
            None => None,
        }
    }

    fn parse_output_mode(arguments: &Vec<&str>) -> Option<OutputMode> {
        let output_mode_flag_position = arguments.iter().position(|arg| *arg == "--output-mode");

        match output_mode_flag_position {
            Some(position) => {
                let output_mode_flag = arguments.get(position + 1).and_then(|s| Some(*s));
                match output_mode_flag {
                    Some("tokens") => Some(OutputMode::Tokens),
                    Some("parse-tree") => Some(OutputMode::ParseTree),
                    Some("ast") => Some(OutputMode::AbstractSyntaxTree),
                    _ => None,
                }
            }
            None => None,
        }
    }
}

#[test]
fn config_parse_output_mode_finds_tokens() {
    let args = vec!["--output-mode", "tokens"];

    let output_mode = Config::parse_output_mode(&args);

    assert_eq!(output_mode.is_some(), true);
    assert_eq!(output_mode.unwrap(), OutputMode::Tokens);
}

#[test]
fn config_parse_output_mode_finds_parse_tree() {
    let args = vec!["--output-mode", "parse-tree"];

    let output_mode = Config::parse_output_mode(&args);

    assert_eq!(output_mode.is_some(), true);
    assert_eq!(output_mode.unwrap(), OutputMode::ParseTree);
}

#[test]
fn config_parse_output_mode_finds_ast() {
    let args = vec!["--output-mode", "ast"];

    let output_mode = Config::parse_output_mode(&args);

    assert_eq!(output_mode.is_some(), true);
    assert_eq!(output_mode.unwrap(), OutputMode::AbstractSyntaxTree);
}

#[test]
fn config_parse_output_mode_expects_correct_position() {
    let args = vec!["--output-mode", "interloper", "tokens"];

    let output_mode = Config::parse_output_mode(&args);

    assert_eq!(output_mode.is_none(), true);
}

#[test]
fn config_parse_output_mode_expects_flag() {
    let args = vec!["tokens"];

    let output_mode = Config::parse_output_mode(&args);

    assert_eq!(output_mode.is_none(), true);
}
