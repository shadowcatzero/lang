use super::Node;
use super::PIdent;
use super::CompilerMsg;
use super::TokenInstance;

impl CompilerMsg {
    pub fn from_instances(instances: &[&TokenInstance], msg: String) -> Self {
        CompilerMsg {
            msg,
            spans: instances.iter().map(|i| i.span).collect(),
        }
    }
    pub fn unexpected_end() -> Self {
        Self::from_msg("unexpected end of input".to_string())
    }
    pub fn identifier_not_found(id: &Node<PIdent>) -> Self {
        Self {
            msg: format!("Identifier '{}' not found", id.as_ref().unwrap()),
            spans: vec![id.origin],
        }
    }
    pub fn unexpected_token(inst: &TokenInstance, expected: &str) -> Self {
        let t = &inst.token;
        CompilerMsg::from_instances(
            &[inst],
            format!("unexpected token {t:?}; expected {expected}"),
        )
    }
}
