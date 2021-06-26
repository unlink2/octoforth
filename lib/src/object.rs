pub type ObjStr = String;
pub type ObjNumber = i64;
pub type ObjReal = f64;
pub type ObjList = Vec<Object>;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Nil,
    Number(ObjNumber),
    Real(ObjReal),
    Str(ObjStr),
    Word(ObjStr)
}
