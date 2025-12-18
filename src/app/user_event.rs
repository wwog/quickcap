use crate::stdio::client::RpcRequest;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum UserEvent {
    Exit,
    RpcMessage(RpcRequest),
}
