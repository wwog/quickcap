use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock, mpsc};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub id: Option<u64>,
}

// 用于内部区分接收到的是 请求 还是 响应
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum IncomingMessage {
    Response(RpcResponse),
    Request(RpcRequest),
}

type RequestCallback = Box<dyn Fn(RpcRequest) + Send + Sync + 'static>;

static INSTANCE: OnceLock<StdRpcClient> = OnceLock::new();

pub struct StdRpcClient {
    next_id: AtomicU64,
    pending_requests: Mutex<HashMap<u64, mpsc::Sender<RpcResponse>>>,
}

impl StdRpcClient {
    pub fn global() -> &'static Self {
        INSTANCE.get().expect("StdRpcClient not initialized")
    }

    /// 发送通知 (Fire & Forget, 不等待响应)
    pub fn send_notification(&self, method: &str, params: serde_json::Value) {
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
            id: None,
        };
        self.write_to_stdout(&req);
    }

    /// 发送响应
    pub fn send_response(&self, id: u64, result: serde_json::Value) {
        let resp = RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: Some(id),
        };
        self.write_to_stdout(&resp);
    }

    /// 发起请求并等待响应 (同步阻塞，带超时)
    pub fn call(
        &self,
        method: &str,
        params: serde_json::Value,
        timeout: Duration,
    ) -> Result<serde_json::Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let (tx, rx) = mpsc::channel();

        {
            let mut map = self.pending_requests.lock().unwrap();
            map.insert(id, tx);
        }

        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
            id: Some(id),
        };
        self.write_to_stdout(&req);

        let result = rx.recv_timeout(timeout);

        {
            let mut map = self.pending_requests.lock().unwrap();
            map.remove(&id);
        }

        match result {
            Ok(resp) => {
                if let Some(err) = resp.error {
                    Err(format!("RPC Error: {:?}", err))
                } else if let Some(res) = resp.result {
                    Ok(res)
                } else {
                    Err("Empty response".to_string())
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => Err("Request timed out".to_string()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err("Channel disconnected".to_string()),
        }
    }

    fn write_to_stdout<T: Serialize>(&self, msg: &T) {
        let mut json = serde_json::to_string(msg).unwrap();
        json.push('\n');
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if let Err(e) = handle.write_all(json.as_bytes()) {
            log::error!("Failed to write to stdout: {}", e);
        }
        let _ = handle.flush();
    }

    /// 初始化 STD RPC 系统
    /// 参数 on_request: 当收到STDIO的请求时，会调用此闭包
    pub fn init<F>(on_request: F)
    where
        F: Fn(RpcRequest) + Send + Sync + 'static,
    {
        let client = StdRpcClient {
            next_id: AtomicU64::new(1),
            pending_requests: Mutex::new(HashMap::new()),
        };

        if INSTANCE.set(client).is_err() {
            panic!("RpcClient already initialized");
        }

        Self::start_background_listener(on_request);
    }
}

impl StdRpcClient {
    fn start_background_listener<F>(on_request: F)
    where
        F: Fn(RpcRequest) + Send + Sync + 'static,
    {
        thread::spawn(move || {
            let stdin = io::stdin();
            let handle = stdin.lock();

            for line in handle.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                let trimmed = line.trim();
                if trimmed.is_empty() || !trimmed.starts_with('{') {
                    continue;
                }

                match serde_json::from_str::<IncomingMessage>(trimmed) {
                    Ok(IncomingMessage::Response(resp)) => {
                        if let Some(id) = resp.id {
                            let client = Self::global();
                            let map = client.pending_requests.lock().unwrap();
                            if let Some(tx) = map.get(&id) {
                                let _ = tx.send(resp);
                            }
                        }
                    }
                    Ok(IncomingMessage::Request(req)) => {
                        on_request(req);
                    }
                    Err(e) => log::error!("Parse error: {}", e),
                }
            }
        });
    }
}
