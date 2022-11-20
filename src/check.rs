use crate::eval::Error;

trait Check {
    fn chck(expected: Rtns, definitions: &mut DefTypes, sym_table: &mut SymbolTable) -> Result<Rtns, Error>;
}

enum Rtns {
    Fallthrough,
    MightReturn(Ty),
    Returns(Ty),
}

enum Ty {
    Int,
    Bool,
    Str,
    Unit,
}

pub struct ArrowType {
    pub return_type: Option<Ty>,
    pub params: Vec<Ty>
}

pub struct DefTypes(HashMap<Ident, ArrowType>);

pub struct SymbolTable {
    pub table: HashMap<Ident, Ty>
}
