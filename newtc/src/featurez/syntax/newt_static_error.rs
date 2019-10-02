#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NewtStaticError {
	ShadowedVariableDeclaration,
	DuplicateVariableDeclaration,
	UndeclaredVariable
}
