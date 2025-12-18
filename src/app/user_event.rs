use crate::stdio::client::RpcRequest;

#[derive(Debug, Clone)]
pub enum UserEvent {
    Exit,
    RpcMessage(RpcRequest),
}
