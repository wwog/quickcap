use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Clone)]
pub struct RpcRequest {
    pub method: String,
    pub params: Option<Value>,
    pub id: Value, // ID 可以是 Number 或 String
}

#[derive(Debug, Clone)]
pub struct RpcNotification {
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct RawMessage {
    jsonrpc: Option<String>,
    method: Option<String>,
    params: Option<Value>,
    result: Option<Value>,
    error: Option<RpcError>,
    id: Option<Value>,
}


static INSTANCE: OnceLock<StdRpcClient> = OnceLock::new();

pub struct StdRpcClient {
    next_id: AtomicU64,
    // 存储发出的请求，等待回复: Map<ID, Sender>
    pending_requests: Mutex<HashMap<String, mpsc::Sender<RpcResponse>>>,
}

impl StdRpcClient {
    pub fn global() -> &'static Self {
        INSTANCE.get().expect("StdRpcClient not initialized! Call init() first.")
    }

    pub fn send_notification(&self, method: &str, params: Option<Value>) {
        // 构建原始 JSON 对象发送，避免定义冗余的 OutgoingNotification 结构
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.write_raw(msg);
    }

    pub fn call(
        &self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value, String> {
        let id_num = self.next_id.fetch_add(1, Ordering::Relaxed);
        let id_str = id_num.to_string(); // 统一转为 String 处理 ID 比较方便

        let (tx, rx) = mpsc::channel();

        // 注册 Pending
        {
            let mut map = self.pending_requests.lock().unwrap();
            map.insert(id_str.clone(), tx);
        }

        // 发送数据
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id_num 
        });
        self.write_raw(msg);

        // 等待响应
        let result = rx.recv_timeout(timeout);

        // 清理 Map
        {
            let mut map = self.pending_requests.lock().unwrap();
            map.remove(&id_str);
        }

        match result {
            Ok(resp) => {
                if let Some(err) = resp.error {
                    Err(format!("RPC Error {}: {}", err.code, err.message))
                } else if let Some(res) = resp.result {
                    Ok(res)
                } else {
                    Ok(Value::Null) // 成功但结果为 null
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => Err("Request timed out".to_string()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err("Channel disconnected".to_string()),
        }
    }

    /// 内部：写 JSON 到 stdout
    fn write_raw(&self, msg: Value) {
        let mut json = serde_json::to_string(&msg).unwrap();
        json.push('\n'); // 必须有换行符
        
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if let Err(e) = handle.write_all(json.as_bytes()) {
            log::error!("Failed to write to stdout: {}", e);
        }
        let _ = handle.flush();
    }

    /// 内部：自动回复响应 (Rust -> Electron)
    fn send_response(&self, id: Value, result: Result<Value, RpcError>) {
        let (res, err) = match result {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(e)),
        };

        let resp = RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: res,
            error: err,
            id,
        };
        
        // 序列化 RpcResponse
        let mut json = serde_json::to_string(&resp).unwrap();
        json.push('\n');

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let _ = handle.write_all(json.as_bytes());
        let _ = handle.flush();
    }

    // --- 初始化函数 ---

    /// 初始化 RPC 系统
    /// 
    /// # 参数
    /// * `on_request`: 处理请求的回调。必须返回 `Result<Value, RpcError>`。系统会自动发送 Response。
    /// * `on_notification`: 处理通知的回调。
    pub fn init<FReq, FNotif>(on_request: FReq, on_notification: FNotif)
    where
        FReq: Fn(RpcRequest) -> Result<Value, RpcError> + Send + Sync + 'static,
        FNotif: Fn(RpcNotification) + Send + Sync + 'static,
    {
        let client = StdRpcClient {
            next_id: AtomicU64::new(1),
            pending_requests: Mutex::new(HashMap::new()),
        };

        if INSTANCE.set(client).is_err() {
            panic!("RpcClient already initialized");
        }

        Self::start_listener(on_request, on_notification);
    }

    fn start_listener<FReq, FNotif>(on_request: FReq, on_notification: FNotif)
    where
        FReq: Fn(RpcRequest) -> Result<Value, RpcError> + Send + Sync + 'static,
        FNotif: Fn(RpcNotification) + Send + Sync + 'static,
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

                match serde_json::from_str::<RawMessage>(trimmed) {
                    Ok(raw) => {
                        let client = Self::global();

                        if let Some(method) = raw.method {
                            // --- 有 Method: 是 请求 或 通知 ---
                            if let Some(id) = raw.id {
                                // A. 有 ID -> 请求 (Request)
                                // 调用回调获取结果
                                let req = RpcRequest { method, params: raw.params, id: id.clone() };
                                let result = on_request(req);
                                // 自动回复
                                client.send_response(id, result);
                            } else {
                                // B. 无 ID -> 通知 (Notification)
                                let notif = RpcNotification { method, params: raw.params };
                                on_notification(notif);
                            }
                        } else if let Some(id) = raw.id {
                            // --- 无 Method 有 ID: 是 响应 (Response) ---
                            // 只有我们主动发起的请求才会收到这种消息
                            // 注意：id 可能是 Number 可能是 String，转 String 查 Map
                            let id_str = match &id {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                _ => id.to_string(),
                            };

                            let map = client.pending_requests.lock().unwrap();
                            if let Some(tx) = map.get(&id_str) {
                                let resp = RpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: raw.result,
                                    error: raw.error,
                                    id,
                                };
                                let _ = tx.send(resp);
                            }
                        } else {
                            log::warn!("Unknown message format: {}", trimmed);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse JSON: {} | Content: {}", e, trimmed);
                    }
                }
            }
        });
    }
}