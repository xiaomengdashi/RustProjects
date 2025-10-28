/// 命令模式实现模块
/// 将MQTT操作封装为命令对象

use std::error::Error;
use crate::client::MqttClient;

/// 命令 trait，定义了所有MQTT命令的通用接口
#[async_trait::async_trait]
pub trait Command {
    async fn execute(&self, client: &mut MqttClient) -> Result<(), Box<dyn Error>>;
    fn get_name(&self) -> &'static str;
}

/// 连接命令
pub struct ConnectCommand {
    addr: String,
}

impl ConnectCommand {
    pub fn new(addr: String) -> Self {
        ConnectCommand { addr }
    }
}

#[async_trait::async_trait]
impl Command for ConnectCommand {
    async fn execute(&self, client: &mut MqttClient) -> Result<(), Box<dyn Error>> {
        client.connect(&self.addr).await
    }
    
    fn get_name(&self) -> &'static str {
        "Connect"
    }
}

/// 断开连接命令
pub struct DisconnectCommand;

impl DisconnectCommand {
    pub fn new() -> Self {
        DisconnectCommand
    }
}

#[async_trait::async_trait]
impl Command for DisconnectCommand {
    async fn execute(&self, client: &mut MqttClient) -> Result<(), Box<dyn Error>> {
        client.disconnect().await
    }
    
    fn get_name(&self) -> &'static str {
        "Disconnect"
    }
}

/// 发布命令
pub struct PublishCommand {
    topic: String,
    message: String,
}

impl PublishCommand {
    pub fn new(topic: String, message: String) -> Self {
        PublishCommand { topic, message }
    }
}

#[async_trait::async_trait]
impl Command for PublishCommand {
    async fn execute(&self, client: &mut MqttClient) -> Result<(), Box<dyn Error>> {
        client.publish(self.topic.clone(), self.message.clone()).await
    }
    
    fn get_name(&self) -> &'static str {
        "Publish"
    }
}

/// 订阅命令
pub struct SubscribeCommand {
    topic: String,
}

impl SubscribeCommand {
    pub fn new(topic: String) -> Self {
        SubscribeCommand { topic }
    }
}

#[async_trait::async_trait]
impl Command for SubscribeCommand {
    async fn execute(&self, client: &mut MqttClient) -> Result<(), Box<dyn Error>> {
        client.subscribe(self.topic.clone()).await
    }
    
    fn get_name(&self) -> &'static str {
        "Subscribe"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_creation() {
        let connect_cmd = ConnectCommand::new("127.0.0.1:1883".to_string());
        assert_eq!(connect_cmd.get_name(), "Connect");
        
        let publish_cmd = PublishCommand::new("test/topic".to_string(), "message".to_string());
        assert_eq!(publish_cmd.get_name(), "Publish");
        
        let subscribe_cmd = SubscribeCommand::new("test/topic".to_string());
        assert_eq!(subscribe_cmd.get_name(), "Subscribe");
    }
}