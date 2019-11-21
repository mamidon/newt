use crate::featurez::syntax::{SyntaxTree, NewtResult, NewtValue, NewtRuntimeError};
use crate::featurez::runtime::scope::Environment;
use crate::featurez::{VirtualMachineState, StrTokenSource};
use crate::featurez::grammar::root_expr;
use crate::featurez::parse::Parser;
use crate::featurez::tokenize;
use std::rc::Rc;

#[test]
fn return_statement_returns_value() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn returns_value() {
		return 42;
	}"#);

	assert_eq_newt_values(42.into(), evaluate(&mut vm, "returns_value()").unwrap());
}

#[test]
fn return_statement_short_circuits() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn return_short_circuits() {
		return 42;
		return 32;
	}"#);

	assert_eq_newt_values(42.into(), evaluate(&mut vm, "return_short_circuits()").unwrap());
}

#[test]
fn return_statement_short_circuit_inside_scope() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn return_early() {
		if (true) {
			return 42;
		}
		return 32;
	}"#);

	assert_eq_newt_values(42.into(), evaluate(&mut vm, "return_early()").unwrap());
}

#[test]
fn return_statement_returns_value_outside_of_function() {
	let mut vm = VirtualMachineState::new();
	let tree: SyntaxTree = "return 42;".into();

	assert_eq_newt_values(42.into(), vm.interpret(&tree).unwrap());
}

#[test]
fn return_statement_no_return_statement_returns_null() {
	let mut vm = VirtualMachineState::new();
	let tree: SyntaxTree = "let x = 1;".into();

	assert_eq!(NewtValue::Null, vm.interpret(&tree).unwrap());
}

#[test]
fn if_statement_executes_correct_branches_for_conditional() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn if_statement(x) {
		if (x) { return 1; }
		return 2;
	}"#);

	assert_eq_newt_values(1.into(), evaluate(&mut vm, "if_statement(true)").unwrap());
	assert_eq_newt_values(2.into(), evaluate(&mut vm, "if_statement(false)").unwrap());
}

#[test]
fn if_else_statement_executes_correct_branches_for_conditional() {
	let mut vm = VirtualMachineState::new();

	define(&mut vm, r#"
	fn if_else_statement(x) {
		if (x) {
			return 1;
		} else {
			return 2;
		}
	}"#);

	assert_eq_newt_values(1.into(), evaluate(&mut vm, "if_else_statement(true)").unwrap());
	assert_eq_newt_values(2.into(), evaluate(&mut vm, "if_else_statement(false)").unwrap());
}

#[test]
fn newt_value_truthy_semantics_for_truthy_values() {
	let mut vm = VirtualMachineState::new();
	define(&mut vm, r#"
	fn truthiness(x) {
		if (x) {
			return true;
		} else {
			return false;
		}
	}"#);

	// integers
	assert_eq_newt_values(true.into(), evaluate(&mut vm, "truthiness(1)").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "truthiness(0)").unwrap());
	assert_eq_newt_values(true.into(), evaluate(&mut vm, "truthiness(-1)").unwrap());

	// booleans
	assert_eq_newt_values(true.into(), evaluate(&mut vm, "truthiness(true)").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "truthiness(false)").unwrap());
}

#[test]
fn newt_value_truthy_semantics_for_untruthy_values() {
	let mut vm = VirtualMachineState::new();
	define(&mut vm, r#"
	fn truthiness(x) {
		if (x) {
			return true;
		} else {
			return false;
		}
	}
	"#);
	// TODO examine why I can't define multiple functions in one define call
	define(&mut vm, r#"
	fn null_value() {}
	"#);

	assert_eq!(NewtRuntimeError::TypeError, evaluate(&mut vm, "truthiness(1.0)").unwrap_err());
	assert_eq!(NewtRuntimeError::TypeError, evaluate(&mut vm, "truthiness(\"foo\")").unwrap_err());
	assert_eq!(NewtRuntimeError::TypeError, evaluate(&mut vm, "truthiness('c')").unwrap_err());
	assert_eq!(NewtRuntimeError::TypeError, evaluate(&mut vm, "truthiness(truthiness)").unwrap_err());
	assert_eq!(NewtRuntimeError::TypeError, evaluate(&mut vm, "truthiness(null_value())").unwrap_err());
}

#[test]
fn newt_value_bool_truthy_semantics() {
	let mut vm = VirtualMachineState::new();

	assert_eq_newt_values(true.into(), evaluate(&mut vm, "true").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "false").unwrap());
}

#[test]
fn newt_value_string_truthy_semantics() {
	let mut vm = VirtualMachineState::new();

	assert_eq_newt_values(false.into(), evaluate(&mut vm, "\"Hello, world!\" == true").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "\"d\" == true").unwrap());
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

fn assert_eq_newt_values(a: NewtValue, b: NewtValue) {
	assert_eq!(a, b);
}
