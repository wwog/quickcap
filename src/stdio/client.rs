#![allow(unused)]

use std::{io::{self, BufRead}, sync::mpsc};

use super::json_rpc::{RpcRequest, RpcResponse};

pub struct StdioClient {}

impl StdioClient {
    /// 启动一个线程，监听标准输入，并处理请求
    pub fn run(&self) {
        // let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let stdin = io::stdin();
            let handle = stdin.lock();

            for line in handle.lines() {
                let Ok(line) = line else {
                    //断开连接
                    break;
                };

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // serde_json::from_str::
            }
        });
    }
}
