#[derive(Debug, Copy, Clone)]
pub enum NewtRuntimeError {
    TypeError,
    UndefinedVariable,
    DuplicateDeclaration,
    // We hit this when a function doesn't return anything, but we try to assign it to a variable
    NullValueEncountered,
}
