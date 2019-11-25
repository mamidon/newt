use crate::featurez::syntax::{SyntaxTree, NewtResult, NewtValue, NewtRuntimeError};
use crate::featurez::runtime::scope::Environment;
use crate::featurez::{VirtualMachine, StrTokenSource};
use crate::featurez::grammar::root_expr;
use crate::featurez::parse::Parser;
use crate::featurez::tokenize;
use std::rc::Rc;

#[test]
fn return_statement_returns_value() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn returns_value() {
		return 42;
	}"#);

	assert_eq_newt_values(42.into(), evaluate(&mut vm, "returns_value()").unwrap());
}

#[test]
fn return_statement_short_circuits() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn return_short_circuits() {
		return 42;
		return 32;
	}"#);

	assert_eq_newt_values(42.into(), evaluate(&mut vm, "return_short_circuits()").unwrap());
}

#[test]
fn return_statement_short_circuit_inside_scope() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
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
	let mut vm = VirtualMachine::new();

	assert_eq_newt_values(42.into(), vm.interpret("return 42;").unwrap());
}

#[test]
fn return_statement_no_return_statement_returns_null() {
	let mut vm = VirtualMachine::new();

	assert_eq!(NewtValue::Null, vm.interpret("let x = 1;").unwrap());
}

#[test]
fn if_statement_executes_correct_branches_for_conditional() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn if_statement(x) {
		if (x) { return 1; }
		return 2;
	}"#);

	assert_eq_newt_values(1.into(), evaluate(&mut vm, "if_statement(true)").unwrap());
	assert_eq_newt_values(2.into(), evaluate(&mut vm, "if_statement(false)").unwrap());
}

#[test]
fn if_statement_uses_truthiness() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn if_statement(x) {
		if (x) { return 1; }
		return 2;
	}"#);

	assert_eq_newt_values(1.into(), evaluate(&mut vm, "if_statement(1)").unwrap());
	assert_eq_newt_values(2.into(), evaluate(&mut vm, "if_statement(0)").unwrap());
}

#[test]
fn if_else_statement_executes_correct_branches_for_conditional() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
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
fn if_else_statement_uses_truthiness() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn if_else_statement(x) {
		if (x) {
			return 1;
		} else {
			return 2;
		}
	}"#);

	assert_eq_newt_values(1.into(), evaluate(&mut vm, "if_else_statement(1)").unwrap());
	assert_eq_newt_values(2.into(), evaluate(&mut vm, "if_else_statement(0)").unwrap());
}

#[test]
fn while_statement_repeats_conditional_evaluations_to_false() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn while_statement(loops_to_do) {
		let loops_done = 0;
		while (loops_to_do > 0) {
			loops_to_do = loops_to_do - 1;
			loops_done = loops_done + 1;
		}

		return loops_done;
	}"#);

	assert_eq_newt_values(10.into(), evaluate(&mut vm, "while_statement(10)").unwrap());
	assert_eq_newt_values(0.into(), evaluate(&mut vm, "while_statement(0)").unwrap());
}

#[test]
fn while_statement_uses_truthy_semantics() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn while_statement(truthy) {
		while (truthy) {
			return true;
		}

		return false;
	}"#);

	assert_eq_newt_values(true.into(), evaluate(&mut vm, "while_statement(1)").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "while_statement(0)").unwrap());
}

#[test]
fn newt_value_truthy_semantics_for_truthy_values() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
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
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
	fn truthiness(x) {
		if (x) {
			return true;
		} else {
			return false;
		}
	}

	fn null_value() {}
	"#);

	assert_eq!(Err(NewtRuntimeError::TypeError), evaluate(&mut vm, "truthiness(1.0)"));
	assert_eq!(Err(NewtRuntimeError::TypeError), evaluate(&mut vm, "truthiness(\"foo\")"));
	assert_eq!(Err(NewtRuntimeError::TypeError), evaluate(&mut vm, "truthiness('c')"));
	assert_eq!(Err(NewtRuntimeError::TypeError), evaluate(&mut vm, "truthiness(truthiness)"));
	assert_eq!(Err(NewtRuntimeError::TypeError), evaluate(&mut vm, "truthiness(null_value())"));
}

#[test]
fn newt_value_bool_equality_semantics() {
	let mut vm = VirtualMachine::new();

	assert_eq_newt_values(true.into(), evaluate(&mut vm, "true == true").unwrap());
	assert_eq_newt_values(true.into(), evaluate(&mut vm, "false == false").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "true == false").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "false == true").unwrap());
}

#[test]
fn newt_value_string_equality_semantics() {
	let mut vm = VirtualMachine::new();

	assert_eq_newt_values(true.into(), evaluate(&mut vm, "\"Hello, world!\" == \"Hello, world!\"").unwrap());
	assert_eq_newt_values(false.into(), evaluate(&mut vm, "\"\" == \"Hello, world!\"").unwrap());
	assert_eq_newt_values(true.into(), evaluate(&mut vm, "\"\" == \"\"").unwrap());
}

#[test]
fn function_declaration_statement_adds_function_to_scope() {
	let mut vm = VirtualMachine::new();

	vm.interpret("fn function_declaration() { return 42; }");

	assert_eq_newt_values(42.into(), evaluate(&mut vm, "function_declaration()").unwrap());
	assert_eq!(Err(NewtRuntimeError::UndefinedVariable), evaluate(&mut vm, "foo()"));
}

#[test]
fn function_declaration_statement_can_nest_declarations() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn outer_declaration() {
		fn inner_declaration() {
			return 32;
		}

		return inner_declaration;
	}"#);

	assert_eq_newt_values(32.into(), evaluate(&mut vm, "outer_declaration()()").unwrap());
}

#[test]
fn function_declaration_statement_can_capture_closure() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn count_to_max(max) {
		let count = 0;
		fn counter() {
			if (count < max) {
				count = count + 1;
			}

			return count;
		}

		return counter;
	}
	let count_to_3 = count_to_max(3);
	"#);

	assert_eq_newt_values(1.into(), evaluate(&mut vm, "count_to_3()").unwrap());
	assert_eq_newt_values(2.into(), evaluate(&mut vm, "count_to_3()").unwrap());
	assert_eq_newt_values(3.into(), evaluate(&mut vm, "count_to_3()").unwrap());
	assert_eq_newt_values(3.into(), evaluate(&mut vm, "count_to_3()").unwrap());
}

#[test]
fn variable_declaration_statement_adds_variable_to_top_scope() {
	let mut vm = VirtualMachine::new();

	vm.interpret("let x = 42;");

	assert_eq_newt_values(42.into(), evaluate(&mut vm, "x").unwrap());
}

#[test]
fn variable_declaration_statement_does_not_effect_scope_prior_to_declaration() {
	let mut vm = VirtualMachine::new();
	let result = vm.interpret(r#"
	let y = x;
	let x = 42;
	"#);

	assert_eq!(Err(NewtRuntimeError::UndefinedVariable), result);
}

#[test]
fn virtual_machine_correctly_computes_fibonacci_5() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
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

fn evaluate(vm: &mut VirtualMachine, source: &str) -> NewtResult {
	let tokens = tokenize(source);
	let token_source = StrTokenSource::new(tokens);
	let mut parser = Parser::new(token_source);

	let completed_parsing = root_expr(parser);
	let tree = SyntaxTree::from_parser(&completed_parsing, source);

	vm.interpret(tree)
}

fn assert_eq_newt_values(a: NewtValue, b: NewtValue) {
	assert_eq!(a, b);
}
