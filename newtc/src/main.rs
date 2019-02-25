#![allow(unused)]
mod parse;

use parse::tokenize;

use std::env::args;
use std::path::PathBuf;

struct Config {
    output_mode: OutputMode,
    entry_file: PathBuf
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum OutputMode {
    Tokens,
    ParseTree,
    AbstractSyntaxTree,
}



fn main() {
    let arguments : Vec<String> = args()
        .collect();
    
    let borrowed_arguments = arguments
        .iter()
        .map(|s| s.as_ref())
        .collect();
    
    let config = Config::parse(&borrowed_arguments);
    
    if let Some(config) = config {
		if config.output_mode == OutputMode::Tokens {
			if let Ok(text) = std::fs::read_to_string(config.entry_file) {
				for token in tokenize(&text) {
					println!("{}", token);
				}
			} else {
				println!("Failed to open file!");
			}
		}
    } else {
        print_help();
    }
}

fn print_help() {
    println!("usage: newtc --entry-file (path) --output-mode (tokens | parse-tree | ast)");
}

impl Config {
    pub fn parse(arguments: &Vec<&str>) -> Option<Config> {
        if let (Some(output_mode), Some(entry_file)) = (Config::parse_output_mode(arguments), Config::parse_entry_file(arguments)) {
            Some(Config {
                output_mode,
                entry_file
            })
        } else {
            None   
        }
    }
    
    fn parse_entry_file(arguments: &Vec<&str>) -> Option<PathBuf> {
        let entry_file_flag_position = arguments
            .iter()
            .position(|arg| *arg == "--entry-file");

        match entry_file_flag_position {
            Some(position) => {
                let entry_file = arguments
                    .get(position + 1)
                    .and_then(|s| Some(PathBuf::from(s)));
                return entry_file;
            },
            None => None
        }
    }
    
    fn parse_output_mode(arguments: &Vec<&str>) -> Option<OutputMode> {
        let output_mode_flag_position = arguments
            .iter()
            .position(|arg| *arg == "--output-mode");
        
        match output_mode_flag_position {
            Some(position) => {
                let output_mode_flag = arguments
                    .get(position + 1)
                    .and_then(|s| Some(*s));
                match output_mode_flag {
                    Some("tokens") => Some(OutputMode::Tokens),
                    Some("parse-tree") => Some(OutputMode::ParseTree),
                    Some("ast") => Some(OutputMode::AbstractSyntaxTree),
                    _ => None 
                }
            },
            None => None
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


