#[derive(Debug, Copy, Clone)]
pub enum NewtRuntimeError {
	TypeError,
	UndefinedVariable,
	DuplicateDeclaration
}
