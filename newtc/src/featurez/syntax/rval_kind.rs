use crate::featurez::syntax::{ObjectPropertyRValNode, VariableRValNode};

pub enum RValKind<'a> {
    VariableRVal(&'a VariableRValNode),
    ObjectPropertyRVal(&'a ObjectPropertyRValNode),
}
