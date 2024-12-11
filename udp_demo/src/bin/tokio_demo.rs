use std::sync::Arc;
use tokio::sync::mpsc;
use lazy_static;
use tokio::sync::broadcast;


// 定义全局 channel
lazy_static::lazy_static! {
    pub static ref BROADCAST_CHANNEL: (
        Arc<tokio::sync::Mutex<mpsc::Sender<String>>>, 
        Arc<tokio::sync::Mutex<mpsc::Receiver<String>>>
    ) = {
        let (tx, rx) = mpsc::channel(100);
        (Arc::new(tokio::sync::Mutex::new(tx)), Arc::new(tokio::sync::Mutex::new(rx)))
    };
}

// 发送消息到 channel
async fn send_message(message: String) {
    let sender = BROADCAST_CHANNEL.0.lock().await;
    match sender.send(message.clone()).await {
        Ok(_) => println!("消息已发送: {}", message),
        Err(e) => eprintln!("发送消息失败: {}", e),
    }
}

// 从 channel 接收消息
async fn receive_messages() {
    let mut receiver = BROADCAST_CHANNEL.1.lock().await;
    while let Some(message) = receiver.recv().await {
        println!("收到消息: {}", message);
    }
}

// 示例使用
async fn example_usage() {
    // 启动接收消息的任务
    let receive_task = tokio::spawn(async {
        receive_messages().await;
    });

    // 发送一些测试消息
    for i in 1..=5 {
        send_message(format!("测试消息 {}", i)).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // 等待接收任务完成
    let _ = receive_task.await;
}

// 在 main 函数中调用示例
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("开始测试 BROADCAST_CHANNEL...");
    
    // 创建关闭信号
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
    let shutdown_tx_clone = shutdown_tx.clone();

    // 设置 Ctrl+C 处理
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            println!("\n收到退出信号，正在关闭服务器...");
            let _ = shutdown_tx_clone.send(());
        }
    });

    // 运行消息处理示例
    tokio::select! {
        _ = example_usage() => {
            println!("消息处理完成");
        }
        _ = shutdown_rx.recv() => {
            println!("程序正在关闭...");
        }
    }
    
    Ok(())
}