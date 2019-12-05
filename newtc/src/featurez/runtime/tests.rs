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

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("returns_value()"));
}

#[test]
fn return_statement_short_circuits() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn return_short_circuits() {
		return 42;
		return 32;
	}"#);

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("return_short_circuits()"));
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

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("return_early()"));
}

#[test]
fn return_statement_returns_value_outside_of_function() {
	let mut vm = VirtualMachine::new();

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("return 42;"));
}

#[test]
fn return_statement_no_return_statement_returns_null() {
	let mut vm = VirtualMachine::new();

	assert_eq!(Ok(NewtValue::Null), vm.interpret("let x = 1;"));
}

#[test]
fn if_statement_executes_correct_branches_for_conditional() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn if_statement(x) {
		if (x) { return 1; }
		return 2;
	}"#);

	assert_eq!(Ok(NewtValue::Int(1)), vm.interpret("if_statement(true)"));
	assert_eq!(Ok(NewtValue::Int(2)), vm.interpret("if_statement(false)"));
}

#[test]
fn if_statement_uses_truthiness() {
	let mut vm = VirtualMachine::new();

	vm.interpret(r#"
	fn if_statement(x) {
		if (x) { return 1; }
		return 2;
	}"#);

	assert_eq!(Ok(NewtValue::Int(1)), vm.interpret("if_statement(1)"));
	assert_eq!(Ok(NewtValue::Int(2)), vm.interpret("if_statement(0)"));
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

	assert_eq!(Ok(NewtValue::Int(1)), vm.interpret("if_else_statement(true)"));
	assert_eq!(Ok(NewtValue::Int(2)), vm.interpret("if_else_statement(false)"));
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

	assert_eq!(Ok(NewtValue::Int(1)), vm.interpret("if_else_statement(1)"));
	assert_eq!(Ok(NewtValue::Int(2)), vm.interpret("if_else_statement(0)"));
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

	assert_eq!(Ok(NewtValue::Int(10)), vm.interpret("while_statement(10)"));
	assert_eq!(Ok(NewtValue::Int(0)), vm.interpret("while_statement(0)"));
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

	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("while_statement(1)"));
	assert_eq!(Ok(NewtValue::Bool(false)), vm.interpret("while_statement(0)"));
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
	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("truthiness(1)"));
	assert_eq!(Ok(NewtValue::Bool(false)), vm.interpret("truthiness(0)"));
	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("truthiness(-1)"));

	// booleans
	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("truthiness(true)"));
	assert_eq!(Ok(NewtValue::Bool(false)), vm.interpret("truthiness(false)"));
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

	assert_eq!(Err(NewtRuntimeError::TypeError), vm.interpret("truthiness(1.0)"));
	assert_eq!(Err(NewtRuntimeError::TypeError), vm.interpret("truthiness(\"foo\")"));
	assert_eq!(Err(NewtRuntimeError::TypeError), vm.interpret("truthiness('c')"));
	assert_eq!(Err(NewtRuntimeError::TypeError), vm.interpret("truthiness(truthiness)"));
	assert_eq!(Err(NewtRuntimeError::TypeError), vm.interpret("truthiness(null_value())"));
}

#[test]
fn newt_value_bool_equality_semantics() {
	let mut vm = VirtualMachine::new();

	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("true == true"));
	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("false == false"));
	assert_eq!(Ok(NewtValue::Bool(false)), vm.interpret("true == false"));
	assert_eq!(Ok(NewtValue::Bool(false)), vm.interpret("false == true"));
}

#[test]
fn newt_value_string_equality_semantics() {
	let mut vm = VirtualMachine::new();

	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("\"Hello, world!\" == \"Hello, world!\""));
	assert_eq!(Ok(NewtValue::Bool(false)), vm.interpret("\"\" == \"Hello, world!\""));
	assert_eq!(Ok(NewtValue::Bool(true)), vm.interpret("\"\" == \"\""));
}

#[test]
fn function_declaration_statement_adds_function_to_scope() {
	let mut vm = VirtualMachine::new();

	vm.interpret("fn function_declaration() { return 42; }");

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("function_declaration()"));
	assert_eq!(Err(NewtRuntimeError::UndefinedVariable), vm.interpret("foo()"));
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

	assert_eq!(Ok(NewtValue::Int(32)), vm.interpret("outer_declaration()()"));
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

	assert_eq!(Ok(NewtValue::Int(1)), vm.interpret("count_to_3()"));
	assert_eq!(Ok(NewtValue::Int(2)), vm.interpret("count_to_3()"));
	assert_eq!(Ok(NewtValue::Int(3)), vm.interpret("count_to_3()"));
	assert_eq!(Ok(NewtValue::Int(3)), vm.interpret("count_to_3()"));
}

#[test]
fn variable_declaration_statement_adds_variable_to_top_scope() {
	let mut vm = VirtualMachine::new();

	vm.interpret("let x = 42;");

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("x"));
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
fn object_literals_are_correctly_evaluated_when_empty() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
	fn empty_object() {
		return {};
	}"#);
	let object = match vm.interpret("empty_object()") {
		Ok(NewtValue::Object(object)) => object,
		_ => panic!("Did not get an object")
	};

	assert_eq!(true, object.borrow().is_empty());
}

#[test]
fn object_literals_are_correctly_evaluated_with_one_property() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
	fn object1() {
		return { x: 42 };
	}"#);
	let object = match vm.interpret("object1()") {
		Ok(NewtValue::Object(object)) => object,
		_ => panic!("Did not get an object")
	};

	assert_eq!(1, object.borrow().len());
	assert_eq!(Some(&NewtValue::Int(42)), object.borrow().get("x"));
}


#[test]
fn object_literals_are_correctly_evaluated_with_multiple_properties() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
	fn object_many() {
		return {
			x: 42,
			y: "hello, world",
			z: 3.14f
		};
	}"#);
	let object = match vm.interpret("object_many()") {
		Ok(NewtValue::Object(object)) => object,
		_ => panic!("Did not get an object")
	};

	assert_eq!(3, object.borrow().len());
	assert_eq!(Some(&NewtValue::Int(42)), object.borrow().get("x"));
	assert_eq!(Some(&NewtValue::String(Rc::new("hello, world".to_string()))), object.borrow().get("y"));
	assert_eq!(Some(&NewtValue::Float(3.14)), object.borrow().get("z"));
}

#[test]
fn object_property_get_fails_with_type_error_for_invalid_property() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
	fn object() {
		return {};
	}
	let instance = object();"#);

	assert_eq!(Err(NewtRuntimeError::UndefinedVariable), vm.interpret("instance.foo"));
}

#[test]
fn object_property_get_returns_expected_value() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
	fn object() {
		return {
			x: 42,
			y: 0
		};
	}
	let instance = object();"#);

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("instance.x"));
}

#[test]
fn object_property_set_saves_expected_value_in_expected_property() {
	let mut vm = VirtualMachine::new();
	vm.interpret(r#"
	fn object() {
		return {
			x: 42,
			y: 0
		};
	}
	let instance = object();
	instance.y = 1;"#);

	assert_eq!(Ok(NewtValue::Int(42)), vm.interpret("instance.x"));
	assert_eq!(Ok(NewtValue::Int(1)), vm.interpret("instance.y"));
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

	assert_eq!(Ok(NewtValue::Int(8)), vm.interpret("fibonacci(6)"));
}

#[test]
fn virtual_machine_short_circuits_incorrect_syntax() {
	let mut vm = VirtualMachine::new();

	assert_eq!(Err(NewtRuntimeError::InvalidSyntaxTree), vm.interpret("2++2"));
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
