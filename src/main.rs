use std::{os::unix::net::UnixListener, sync::Arc, thread};

use dashmap::DashSet;

use crate::ta_server::handle_ta_request;

mod protocal;
mod ta_server;

fn main() -> anyhow::Result<()> {
    let path = "/tmp/server.sock";

    // 如果存在旧 socket 文件先删掉
    let _ = std::fs::remove_file(path);

    let listener = UnixListener::bind(path)?;
    println!("Server running at {}", path);

    // 注册表：记录 app_name -> 注册时间
    let registry = Arc::new(DashSet::<String>::new());

    loop {
        let (stream, _) = listener.accept()?;
        let registry = registry.clone();
        thread::spawn(move || handle_ta_request(stream, registry));
    }
}
