use crate::featurez::syntax::{VariableRValNode, ObjectPropertyRValNode};

pub enum RValKind<'a> {
	VariableRVal(&'a VariableRValNode),
	ObjectPropertyRVal(&'a ObjectPropertyRValNode)
}
