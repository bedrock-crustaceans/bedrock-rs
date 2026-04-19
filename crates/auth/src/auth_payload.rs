#[derive(Clone, Debug)]
pub enum AuthPayload {
    Chain(Vec<String>),
    Token(String),
}
