#![allow(unused)]

#[macro_use]
extern crate lazy_static;

mod featurez;

use crate::featurez::*;

use std::env::args;
use std::io::{stdin, stdout};
use std::io::{Error, Write};
use std::path::PathBuf;
use std::str::Chars;

struct Config {
    display_tokenization: bool,
    display_parsing: bool,
    display_evaluation: bool,
    display_help: bool,
    entry_file: Option<PathBuf>,
}

fn main() {
    let arguments: Vec<String> = args().collect();

    let borrowed_arguments = arguments.iter().map(|s| s.as_ref()).collect();

    let config = Config::parse(&borrowed_arguments);

    if config.display_help {
        print_help();
        return;
    }

    let mut vm = VirtualMachine::new();

    if let Some(pathname) = &config.entry_file {
        let file_contents = std::fs::read_to_string(pathname).ok();
        process_input(&config, &file_contents.unwrap(), &mut vm);
    }

    repl(&config, &mut vm);
}

fn process_input(config: &Config, input: &str, vm: &mut VirtualMachine) {
    let tokens = tokenize(input);

    if config.display_tokenization {
        println!("{:?}\n", tokens);
    }

    let tree: SyntaxTree = input.into();

    if config.display_parsing {
        println!("{}\n", tree);
    }

    let evaluation = vm.interpret(tree);

    if config.display_evaluation {
        println!("{:?}\n", evaluation);
    }
}

fn repl(config: &Config, vm: &mut VirtualMachine) {
    let mut input_buffer = String::new();

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
            process_input(&config, &input_buffer, vm);
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

fn print_help() {
    println!("usage: newtc [--entry-file (path)] [--tokens] [--parse] [--no-eval] [--help]");
}

impl Config {
    pub fn parse(arguments: &Vec<&str>) -> Config {
        let tokens_flag = Config::parse_tokens_flag(arguments);
        let parse_flag = Config::parse_parse_flag(arguments);
        let no_eval_flag = Config::parse_no_eval_flag(arguments);
        let help_flag = Config::parse_help_flag(arguments);
        let entry_file = Config::parse_entry_file(arguments);

        Config {
            display_tokenization: tokens_flag,
            display_parsing: parse_flag,
            display_evaluation: !no_eval_flag,
            display_help: help_flag,
            entry_file,
        }
    }

    fn parse_tokens_flag(arguments: &Vec<&str>) -> bool {
        arguments.contains(&"--tokens")
    }

    fn parse_parse_flag(arguments: &Vec<&str>) -> bool {
        arguments.contains(&"--parse")
    }

    fn parse_no_eval_flag(arguments: &Vec<&str>) -> bool {
        arguments.contains(&"--no-eval")
    }

    fn parse_help_flag(arguments: &Vec<&str>) -> bool {
        arguments.contains(&"--help")
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
}

#[test]
fn config_parse_output_mode_finds_tokens_flag() {
    let args = vec!["--tokens"];

    let config = Config::parse(&args);

    assert_eq!(true, config.display_tokenization);
    assert_eq!(false, config.display_parsing);
    assert_eq!(true, config.display_evaluation);
    assert_eq!(false, config.display_help);
    assert_eq!(None, config.entry_file);
}

#[test]
fn config_parse_output_mode_finds_parse_flag() {
    let args = vec!["--parse"];

    let config = Config::parse(&args);

    assert_eq!(false, config.display_tokenization);
    assert_eq!(true, config.display_parsing);
    assert_eq!(true, config.display_evaluation);
    assert_eq!(false, config.display_help);
    assert_eq!(None, config.entry_file);
}

#[test]
fn config_parse_output_mode_finds_eval_flag() {
    let args = vec!["--no-eval"];

    let config = Config::parse(&args);

    assert_eq!(false, config.display_tokenization);
    assert_eq!(false, config.display_parsing);
    assert_eq!(false, config.display_evaluation);
    assert_eq!(false, config.display_help);
    assert_eq!(None, config.entry_file);
}

#[test]
fn config_parse_output_mode_finds_help_flag() {
    let args = vec!["--help"];

    let config = Config::parse(&args);

    assert_eq!(false, config.display_tokenization);
    assert_eq!(false, config.display_parsing);
    assert_eq!(true, config.display_evaluation);
    assert_eq!(true, config.display_help);
    assert_eq!(None, config.entry_file);
}

#[test]
fn config_parse_output_mode_finds_all_flags() {
    let args = vec!["--help", "--tokens", "--parse"];

    let config = Config::parse(&args);

    assert_eq!(true, config.display_tokenization);
    assert_eq!(true, config.display_parsing);
    assert_eq!(true, config.display_evaluation);
    assert_eq!(true, config.display_help);
    assert_eq!(None, config.entry_file);
}

#[test]
fn config_parse_entry_file_finds_pathname() {
    let args = vec!["--entry-file", "pathname"];

    let config = Config::parse(&args);

    assert_eq!(false, config.display_tokenization);
    assert_eq!(false, config.display_parsing);
    assert_eq!(true, config.display_evaluation);
    assert_eq!(false, config.display_help);
    assert_eq!(Some("pathname"), config.entry_file.unwrap().to_str());
}

#[test]
fn config_parse_output_mode_finds_all_flags_and_entry_file() {
    let args = vec![
        "--help",
        "--tokens",
        "--parse",
        "--no-eval",
        "--entry-file",
        "pathname",
    ];

    let config = Config::parse(&args);

    assert_eq!(true, config.display_tokenization);
    assert_eq!(true, config.display_parsing);
    assert_eq!(false, config.display_evaluation);
    assert_eq!(true, config.display_help);
    assert_eq!(Some("pathname"), config.entry_file.unwrap().to_str());
}
