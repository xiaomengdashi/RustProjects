use std::error::Error;

mod protocol;
mod client;
mod packet;
mod server;
mod patterns;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志系统
    env_logger::init();
    
    log::info!("Rust Algorithmic Network Protocol - MQTT Implementation");
    
    // 启动服务端和客户端的选项
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "server" => {
                start_server().await?;
            },
            "client" => {
                start_client().await?;
            },
            _ => {
                log::warn!("Usage: {} [server|client]", args[0]);
            }
        }
    } else {
        log::info!("Usage: {} [server|client]", args[0]);
        log::info!("Starting client by default...");
        start_client().await?;
    }
    
    Ok(())
}

async fn start_server() -> Result<(), Box<dyn Error>> {
    log::info!("Starting MQTT broker on port 1883...");
    server::run_broker("127.0.0.1:1883").await?;
    Ok(())
}

async fn start_client() -> Result<(), Box<dyn Error>> {
    log::info!("Starting MQTT client...");
    
    // 示例：创建MQTT客户端并连接到测试服务器
    let mut client = client::MqttClient::new("test-client".to_string());
    
    // 注册消息回调
    client.on_message("test/topic".to_string(), |topic, message| {
        log::info!("Received message on {}: {}", topic, message);
    }).await;
    
    // 使用命令模式执行操作
    let connect_cmd = patterns::ConnectCommand::new("127.0.0.1:1883".to_string());
    client.execute_command(&connect_cmd).await?;
    
    let subscribe_cmd = patterns::SubscribeCommand::new("test/topic".to_string());
    client.execute_command(&subscribe_cmd).await?;
    
    let publish_cmd = patterns::PublishCommand::new("test/topic".to_string(), "Hello, MQTT with Command Pattern!".to_string());
    client.execute_command(&publish_cmd).await?;
    
    // 启动消息监听循环（在实际应用中，这里可能需要在单独的任务中运行）
    // client.start_listening().await?;
    
    // 演示断开连接
    let disconnect_cmd = patterns::DisconnectCommand::new();
    client.execute_command(&disconnect_cmd).await?;
    
    log::info!("MQTT client demo completed");
    Ok(())
}