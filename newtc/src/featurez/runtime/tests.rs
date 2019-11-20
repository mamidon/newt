use crate::featurez::syntax::{SyntaxTree, NewtResult, NewtValue};
use crate::featurez::runtime::scope::Environment;
use crate::featurez::{VirtualMachineState, StrTokenSource};
use crate::featurez::grammar::root_expr;
use crate::featurez::parse::Parser;
use crate::featurez::tokenize;

#[test]
fn return_statements_return_values() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn return_works() {
		return 42;
	}"#);

	assert_eq!(Ok(NewtValue::Int(42)), evaluate(&mut vm, "return_works()"));
}

#[test]
fn return_statements_short_circuit() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn return_early() {
		return 42;
		return 32;
	}"#);

	assert_eq!(Ok(NewtValue::Int(42)), evaluate(&mut vm, "return_early()"));
}

#[test]
fn return_statements_short_circuit_inside_scope() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn return_early() {
		if (true) {
			return 42;
		}
		return 32;
	}"#);

	assert_eq!(Ok(NewtValue::Int(42)), evaluate(&mut vm, "return_early()"));
}

#[test]
fn if_statements_do_not_execute_when_condition_is_falsey() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn fibonacci_step(x) {
		if (x == 2) { return 1; }
		if (x <= 1) { return 1; }

		return x;
	}"#);

	assert_eq!(NewtValue::Int(10), evaluate(&mut vm, "fibonacci_step(10)").unwrap());
	assert_eq!(NewtValue::Int(1), evaluate(&mut vm, "fibonacci_step(2)").unwrap());
	assert_eq!(NewtValue::Int(1), evaluate(&mut vm, "fibonacci_step(1)").unwrap());
	assert_eq!(NewtValue::Int(1), evaluate(&mut vm, "fibonacci_step(0)").unwrap());
}

#[test]
fn virtual_machine_correctly_computes_fibonacci_5() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
		fn fibonacci(x) {
			if (x == 2) {
				return 1;
			}
			if (x == 1) {
				return 1;
			}
			if (x == 0) {
				return 0;
			}

			return fibonacci(x-2) + fibonacci(x-1);
		}"#);

	assert_eq!(Ok(NewtValue::Int(8)), evaluate(&mut vm, "fibonacci(6)"));
}

fn define(vm: &mut VirtualMachineState, source: &str) {
	let tree: SyntaxTree = source.into();
	let result = vm.interpret(&tree);

	assert_eq!(NewtValue::Null, result.unwrap());
}

fn evaluate(vm: &mut VirtualMachineState, source: &str) -> NewtResult {
	let tokens = tokenize(source);
	let token_source = StrTokenSource::new(tokens);
	let mut parser = Parser::new(token_source);

	let completed_parsing = root_expr(parser);
	let tree = SyntaxTree::from_parser(&completed_parsing, source);

	vm.interpret(&tree)
}
