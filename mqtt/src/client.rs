/// MQTT客户端实现
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::packet::ConnectPacket;
use crate::protocol::PacketType;
use bytes::{BytesMut, BufMut};
use crate::patterns::{Command, State, ClientState, DisconnectedState, ConnectedState};
use crate::patterns::states::{ConnectingState, DisconnectingState};

// 消息回调类型
type MessageCallback = Box<dyn Fn(String, String) + Send + Sync>;

pub struct MqttClient {
    client_id: String,
    stream: Option<TcpStream>,
    subscriptions: Arc<Mutex<HashMap<String, Vec<MessageCallback>>>>,
    state: Box<dyn State + Send>,
}

impl MqttClient {
    pub fn new(client_id: String) -> Self {
        MqttClient {
            client_id,
            stream: None,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            state: Box::new(DisconnectedState),
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> ClientState {
        self.state.get_state()
    }

    /// 设置状态
    fn set_state(&mut self, state: Box<dyn State + Send>) {
        self.state = state;
    }
    
    /// 状态转换
    fn transition_to(&mut self, target_state: ClientState) {
        let target_state_copy = target_state.clone(); // 创建副本用于日志记录
        if let Some(new_state) = self.state.transition_to(target_state) {
            log::debug!("Transitioning from {:?} to {:?}", self.state.get_state(), new_state.get_state());
            self.state = new_state;
        } else {
            log::warn!("Invalid state transition from {:?} to {:?}", self.state.get_state(), target_state_copy);
        }
    }

    /// 执行命令
    pub async fn execute_command(&mut self, command: &dyn Command) -> Result<(), Box<dyn std::error::Error>> {
        // 检查当前状态是否允许执行该命令
        if !self.state.can_execute_command(command.get_name()) {
            return Err(format!("Cannot execute {} command in {:?} state", 
                              command.get_name(), self.state.get_state()).into());
        }

        // 执行命令
        let result = command.execute(self).await;
        
        // 根据命令类型和执行结果更新状态
        match command.get_name() {
            "Connect" => {
                if result.is_ok() {
                    self.set_state(Box::new(ConnectedState));
                }
            },
            _ => {}
        }
        
        result
    }

    /// 连接到MQTT代理
    pub async fn connect(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 检查当前状态是否允许连接
        if !self.state.can_execute_command("Connect") {
            return Err(format!("Cannot connect in {:?} state", self.state.get_state()).into());
        }
        
        // 进入连接中状态
        self.transition_to(ClientState::Connecting);
        
        let stream_result = TcpStream::connect(addr).await;
        
        match stream_result {
            Ok(stream) => {
                self.stream = Some(stream);
                
                // 创建并发送CONNECT包
                let connect_packet = ConnectPacket::new(self.client_id.clone());
                let encoded_packet = connect_packet.encode();
                
                log::debug!("Sending CONNECT packet: {:02x?}", encoded_packet);
                
                if let Some(ref mut stream) = self.stream {
                    stream.write_all(&encoded_packet).await?;
                    
                    // 读取CONNACK响应（最大4字节）
                    let mut buffer = [0u8; 4];
                    match stream.read(&mut buffer).await {
                        Ok(n) => {
                            if n > 0 {
                                log::debug!("Received {} bytes for CONNACK: {:02x?}", n, &buffer[..n]);
                                
                                // 验证CONNACK包类型
                                let packet_type = (buffer[0] >> 4) as u8;
                                if packet_type == PacketType::CONNACK as u8 {
                                    // 解析CONNACK包
                                    if n >= 4 {
                                        let ack_flags = buffer[2];
                                        let return_code = buffer[3];
                                        log::info!("Connected to MQTT broker at {}, ack_flags: {:02x}, return_code: {:02x}", 
                                            addr, ack_flags, return_code);
                                        
                                        match return_code {
                                            0x00 => log::info!("CONNACK: Connection accepted"),
                                            0x01 => log::error!("CONNACK: Unacceptable protocol version"),
                                            0x02 => log::error!("CONNACK: Identifier rejected"),
                                            0x03 => log::error!("CONNACK: Server unavailable"),
                                            0x04 => log::error!("CONNACK: Bad user name or password"),
                                            0x05 => log::error!("CONNACK: Not authorized"),
                                            _ => log::error!("CONNACK: Unknown return code: {:02x}", return_code),
                                        }
                                        
                                        if return_code == 0x00 {
                                            // 连接成功，更新状态
                                            self.transition_to(ClientState::Connected);
                                            Ok(())
                                        } else {
                                            // 连接失败，回到断开连接状态
                                            self.transition_to(ClientState::Disconnected);
                                            Err("Connection rejected by broker".into())
                                        }
                                    } else {
                                        log::error!("CONNACK packet too short: {} bytes", n);
                                        self.transition_to(ClientState::Disconnected);
                                        Err("Failed to connect: Invalid response".into())
                                    }
                                } else {
                                    log::error!("Invalid CONNACK response, expected packet type {:?}, got {}", 
                                        PacketType::CONNACK, packet_type);
                                    self.transition_to(ClientState::Disconnected);
                                    Err("Failed to connect: Invalid response".into())
                                }
                            } else {
                                log::error!("No data received for CONNACK");
                                self.transition_to(ClientState::Disconnected);
                                Err("Failed to connect: No response".into())
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to read CONNACK: {}", e);
                            self.transition_to(ClientState::Disconnected);
                            Err(e.into())
                        }
                    }
                } else {
                    self.transition_to(ClientState::Disconnected);
                    Err("Failed to connect: No stream".into())
                }
            },
            Err(e) => {
                log::error!("Failed to connect to {}: {}", addr, e);
                // 连接失败，回到断开连接状态
                self.transition_to(ClientState::Disconnected);
                Err(e.into())
            }
        }
    }

    /// 订阅主题
    pub async fn subscribe(&mut self, topic: String) -> Result<(), Box<dyn std::error::Error>> {
        // 检查当前状态是否允许订阅
        if !self.state.can_execute_command("Subscribe") {
            return Err(format!("Cannot subscribe in {:?} state", self.state.get_state()).into());
        }
        
        // 先创建包，避免借用冲突
        let packet = Self::create_subscribe_packet(&topic);
        
        log::debug!("Sending SUBSCRIBE packet: {:02x?}", packet);
        
        if let Some(ref mut stream) = self.stream {
            stream.write_all(&packet).await?;
            
            // 读取响应 (最大20字节，给一些余量)
            let mut buffer = [0u8; 20];
            match stream.read(&mut buffer).await {
                Ok(n) => {
                    if n > 0 {                       
                        // 使用公共解析方法
                        if let Some((packet_type, _payload)) = Self::parse_packet(&buffer[..n]) {
                            log::debug!("Received packet type: {}", packet_type);
                            
                            match packet_type {
                                9 => {
                                    // SUBACK包
                                    if n >= 3 {
                                        let packet_id = ((buffer[2] as u16) << 8) | (buffer[3] as u16);
                                        log::info!("Subscribed to topic: {}, packet_id: {}", topic, packet_id);
                                        
                                        // 检查返回码（第5个字节开始）
                                        if n >= 5 {
                                            let return_code = buffer[4];
                                            match return_code {
                                                0x00 => log::info!("SUBACK: QoS 0 granted"),
                                                0x01 => log::info!("SUBACK: QoS 1 granted"),
                                                0x02 => log::info!("SUBACK: QoS 2 granted"),
                                                0x80 => log::warn!("SUBACK: Subscription failed"),
                                                _ => log::warn!("SUBACK: Unknown return code: {:02x}", return_code),
                                            }
                                        }
                                    } else {
                                        log::warn!("SUBACK packet too short: {} bytes", n);
                                    }
                                    Ok(())
                                },
                                3 => {
                                    // PUBLISH包 - 这可能是我们订阅的主题上已经存在的消息
                                    log::info!("Received PUBLISH packet as response to SUBSCRIBE");
                                    // 这里我们可以选择继续等待SUBACK，或者认为这是有效的响应
                                    // 为简单起见，我们将其视为订阅成功
                                    log::info!("Subscribed to topic: {} (received PUBLISH response)", topic);
                                    Ok(())
                                },
                                _ => {
                                    log::error!("Unexpected response packet type: {}, expected SUBACK (9)", packet_type);
                                    Err("Failed to subscribe: Unexpected response".into())
                                }
                            }
                        } else {
                            log::error!("Failed to parse response packet");
                            Err("Failed to subscribe: Invalid response".into())
                        }
                    } else {
                        log::error!("No data received for SUBACK");
                        Err("Failed to subscribe: No response".into())
                    }
                }
                Err(e) => {
                    log::error!("Failed to read response: {}", e);
                    Err(e.into())
                }
            }
        } else {
            Err("Not connected to broker".into())
        }
    }

    /// 发布消息
    pub async fn publish(&mut self, topic: String, message: String) -> Result<(), Box<dyn std::error::Error>> {
        // 检查当前状态是否允许发布
        if !self.state.can_execute_command("Publish") {
            return Err(format!("Cannot publish in {:?} state", self.state.get_state()).into());
        }
        
        let packet = Self::create_publish_packet(&topic, &message);
        
        log::debug!("Sending PUBLISH packet: {:02x?}", packet);
        
        if let Some(ref mut stream) = self.stream {
            stream.write_all(&packet).await?;
            Ok(())
        } else {
            Err("Not connected to broker".into())
        }
    }
    
    /// 创建SUBSCRIBE包
    fn create_subscribe_packet(topic: &str) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        
        // 固定头部 - SUBSCRIBE包类型和标志
        let header = (PacketType::SUBSCRIBE as u8) << 4 | 0b0010; // SUBSCRIBE包需要设置第1、2位为0010
        buffer.put_u8(header);
        
        // 主题长度和主题
        let topic_bytes = topic.as_bytes();
        // 包标识符(2) + 主题长度字段(2) + 主题 + QoS(1)
        let payload_length = 2 + 2 + topic_bytes.len() + 1;
        
        // 剩余长度编码（可变长度编码）
        Self::encode_remaining_length_static(&mut buffer, payload_length);
        
        // 包标识符（简化处理，固定为1）
        buffer.put_u16(1);
        
        // 主题长度
        buffer.put_u16(topic_bytes.len() as u16);
        
        // 主题
        buffer.extend_from_slice(topic_bytes);
        
        // QoS等级（0）
        buffer.put_u8(0);
        
        let result = buffer.to_vec();
        log::debug!("Created SUBSCRIBE packet: {:02x?}", result);
        result
    }
    
    /// 编码剩余长度（使用可变长度编码）
    fn encode_remaining_length_static(buffer: &mut BytesMut, length: usize) {
        let mut remaining_length = length;
        loop {
            let mut digit = remaining_length % 128;
            remaining_length /= 128;
            if remaining_length > 0 {
                digit |= 128;
            }
            buffer.put_u8(digit as u8);
            if remaining_length == 0 {
                break;
            }
        }
    }

    /// 创建PUBLISH包
    fn create_publish_packet(topic: &str, message: &str) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        
        // 固定头部 - PUBLISH包类型和标志 (QoS 0)
        let header = (PacketType::PUBLISH as u8) << 4;
        buffer.put_u8(header);
        
        // 主题长度和主题，消息内容
        let topic_bytes = topic.as_bytes();
        let message_bytes = message.as_bytes();
        let payload_length = 2 + topic_bytes.len() + message_bytes.len();
        
        // 剩余长度编码（可变长度编码）
        Self::encode_remaining_length_static(&mut buffer, payload_length);
        
        // 主题长度
        buffer.put_u16(topic_bytes.len() as u16);
        
        // 主题
        buffer.extend_from_slice(topic_bytes);
        
        // 消息内容
        buffer.extend_from_slice(message_bytes);
        
        let result = buffer.to_vec();
        log::debug!("Created PUBLISH packet: {:02x?}", result);
        result
    }

    /// 注册消息回调
    pub async fn on_message<F>(&mut self, topic: String, callback: F) 
    where 
        F: Fn(String, String) + Send + Sync + 'static
    {
        let mut subs = self.subscriptions.lock().await;
        if let Some(callbacks) = subs.get_mut(&topic) {
            callbacks.push(Box::new(callback));
        } else {
            subs.insert(topic, vec![Box::new(callback)]);
        }
    }
    
    /// 启动消息监听循环
    pub async fn start_listening(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Starting message listening loop...");
        
        if let Some(ref mut stream) = self.stream {
            let mut buffer = [0u8; 1024];
            
            loop {
                match stream.read(&mut buffer).await {
                    Ok(n) => {
                        if n > 0 {                            
                            // 使用公共解析方法
                            if let Some((packet_type, payload)) = Self::parse_packet(&buffer[..n]) {
                                log::debug!("Received packet type: {}", packet_type);
                                
                                match packet_type {
                                    3 => {
                                        // PUBLISH包
                                        if let Some((topic, message)) = Self::extract_publish_payload(&payload) {
                                            log::info!("Received PUBLISH message on topic '{}': {}", topic, message);
                                            
                                            // 触发回调
                                            let subs = self.subscriptions.lock().await;
                                            if let Some(callbacks) = subs.get(&topic) {
                                                for callback in callbacks {
                                                    callback(topic.clone(), message.clone());
                                                }
                                            }
                                        }
                                    },
                                    _ => {
                                        log::debug!("Received unhandled packet type: {}", packet_type);
                                    }
                                }
                            }
                        } else {
                            log::info!("Connection closed by server");
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading from socket: {}", e);
                        return Err(e.into());
                    }
                }
            }
        } else {
            return Err("Not connected to broker".into());
        }
        
        Ok(())
    }
    
    /// 解析MQTT包
    fn parse_packet(data: &[u8]) -> Option<(u8, Vec<u8>)> {
        if data.is_empty() {
            return None;
        }
        
        // 解析包类型
        let packet_type = (data[0] >> 4) as u8;
        
        // 解析剩余长度（简化处理，假设长度<128）
        if data.len() < 2 {
            return None;
        }
        
        let remaining_length = data[1] as usize;
        if data.len() < 2 + remaining_length {
            return None;
        }
        
        // 提取载荷
        let payload = data[2..2 + remaining_length].to_vec();
        
        Some((packet_type, payload))
    }
    
    /// 从PUBLISH包载荷中提取主题和消息
    fn extract_publish_payload(payload: &[u8]) -> Option<(String, String)> {
        if payload.len() < 4 {
            return None;
        }
        
        // 解析主题长度
        let topic_len = ((payload[0] as u16) << 8 | (payload[1] as u16)) as usize;
        
        // 检查缓冲区长度
        if payload.len() < 2 + topic_len {
            return None;
        }
        
        // 解析主题
        let topic = String::from_utf8_lossy(&payload[2..2 + topic_len]).to_string();
        
        // 解析消息内容
        let message_start = 2 + topic_len;
        if message_start >= payload.len() {
            return None;
        }
        
        let message = String::from_utf8_lossy(&payload[message_start..]).to_string();
        
        Some((topic, message))
    }
    
    /// 断开与MQTT代理的连接
    pub async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 检查当前状态是否允许断开连接
        if !self.state.can_execute_command("Disconnect") {
            return Err(format!("Cannot disconnect in {:?} state", self.state.get_state()).into());
        }
        
        // 进入断开连接中状态
        self.transition_to(ClientState::Disconnecting);
        
        // 发送DISCONNECT包
        let disconnect_packet = Self::create_disconnect_packet();
        log::debug!("Sending DISCONNECT packet: {:02x?}", disconnect_packet);
        
        if let Some(ref mut stream) = self.stream {
            match stream.write_all(&disconnect_packet).await {
                Ok(_) => {
                    log::info!("Sent DISCONNECT packet to broker");
                },
                Err(e) => {
                    log::error!("Failed to send DISCONNECT packet: {}", e);
                }
            }
            
            // 关闭连接
            match stream.shutdown().await {
                Ok(_) => {
                    log::info!("Connection closed successfully");
                },
                Err(e) => {
                    log::error!("Error closing connection: {}", e);
                }
            }
        }
        
        // 清理连接
        self.stream = None;
        
        // 进入断开连接状态
        self.transition_to(ClientState::Disconnected);
        
        Ok(())
    }
    
    /// 创建DISCONNECT包
    fn create_disconnect_packet() -> Vec<u8> {
        let mut buffer = Vec::new();
        
        // 固定头部 - DISCONNECT包类型
        let header = (PacketType::DISCONNECT as u8) << 4;
        buffer.push(header);
        
        // 剩余长度为0
        buffer.push(0x00);
        
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_subscribe_packet() {
        let topic = "test/topic";
        let packet = MqttClient::create_subscribe_packet(topic);
        
        // 验证包不为空
        assert!(!packet.is_empty());
        // 验证第一个字节是SUBSCRIBE包类型
        assert_eq!(packet[0] >> 4, PacketType::SUBSCRIBE as u8);
    }
    
    #[test]
    fn test_create_publish_packet() {
        let topic = "test/topic";
        let message = "Hello, MQTT!";
        let packet = MqttClient::create_publish_packet(topic, message);
        
        // 验证包不为空
        assert!(!packet.is_empty());
        // 验证第一个字节是PUBLISH包类型
        assert_eq!(packet[0] >> 4, PacketType::PUBLISH as u8);
    }
    
    #[test]
    fn test_encode_remaining_length_static() {
        let mut buffer = BytesMut::new();
        MqttClient::encode_remaining_length_static(&mut buffer, 127);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0], 127);
        
        let mut buffer = BytesMut::new();
        MqttClient::encode_remaining_length_static(&mut buffer, 128);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], 128 | 128); // 继续位 + 0
        assert_eq!(buffer[1], 1); // 1
    }
    
    #[test]
    fn test_parse_packet() {
        // 创建一个简单的CONNECT包用于测试
        let packet_data = vec![0x10, 0x00]; // CONNECT包类型 + 剩余长度0
        let result = MqttClient::parse_packet(&packet_data);
        
        assert!(result.is_some());
        let (packet_type, payload) = result.unwrap();
        assert_eq!(packet_type, PacketType::CONNECT as u8);
        assert!(payload.is_empty());
    }
    
    #[test]
    fn test_extract_publish_payload() {
        // 创建一个简单的PUBLISH载荷用于测试
        let mut payload = vec![0x00, 0x05]; // 主题长度: 5
        payload.extend_from_slice(b"topic"); // 主题: "topic"
        payload.extend_from_slice(b"message"); // 消息: "message"
        
        let result = MqttClient::extract_publish_payload(&payload);
        
        assert!(result.is_some());
        let (topic, message) = result.unwrap();
        assert_eq!(topic, "topic");
        assert_eq!(message, "message");
    }
}