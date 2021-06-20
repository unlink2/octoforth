#[derive(PartialEq, Eq, Clone)]
pub enum Object {
    Nil,
    Number(i64),
    Regex(String)
}
