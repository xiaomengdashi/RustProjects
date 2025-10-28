/// MQTT服务端实现
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::error::Error;
use rand;

// 存储订阅信息的类型
type Subscriptions = Arc<Mutex<HashMap<String, Vec<String>>>>;

pub struct MqttBroker {
    listener: TcpListener,
    subscriptions: Subscriptions,
    // 使用broadcast来发送消息给所有订阅者
    tx: broadcast::Sender<(String, String)>,
}

impl MqttBroker {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        let subscriptions: Subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let (tx, _rx) = broadcast::channel(100);
        
        Ok(MqttBroker {
            listener,
            subscriptions,
            tx,
        })
    }

    /// 运行MQTT代理服务器
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        log::info!("MQTT Broker listening on {}", self.listener.local_addr()?);
        
        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    log::info!("New client connected: {}", addr);
                    
                    // 为每个客户端创建一个处理任务
                    let subscriptions = self.subscriptions.clone();
                    let tx = self.tx.clone();
                    let rx = tx.subscribe();
                    
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(socket, subscriptions, tx, rx).await {
                            log::error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => log::error!("Error accepting connection: {}", e),
            }
        }
    }
}

/// 处理单个客户端连接
async fn handle_client(
    mut socket: TcpStream,
    subscriptions: Subscriptions,
    tx: broadcast::Sender<(String, String)>,
    mut rx: broadcast::Receiver<(String, String)>,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 1024];
    let mut client_id = String::new();
    
    loop {
        tokio::select! {
            // 处理来自客户端的消息
            result = socket.read(&mut buf) => {
                match result {
                    Ok(0) => {
                        // 客户端断开连接
                        log::info!("Client disconnected");
                        return Ok(());
                    }
                    Ok(n) => {
                        // 解析MQTT包
                        if let Some(packet_type) = parse_packet_type(buf[0]) {
                            match packet_type {
                                1 => {
                                    // CONNECT包
                                    client_id = handle_connect(&mut socket, &buf[..n]).await?;
                                    log::info!("Client connected with ID: {}", client_id);
                                },
                                3 => {
                                    // PUBLISH包
                                    if let Some((topic, message)) = parse_publish(&buf[..n]) {
                                        log::info!("Publishing message to topic '{}': {}", topic, message);
                                        // 广播消息给所有订阅者
                                        let _ = tx.send((topic.clone(), message.clone()));
                                    }
                                },
                                8 => {
                                    // SUBSCRIBE包
                                    if let Some(topic) = parse_subscribe(&buf[..n]) {
                                        log::info!("Client subscribed to topic: {}", topic);
                                        // 添加订阅
                                        let mut subs = subscriptions.lock().await;
                                        if let Some(clients) = subs.get_mut(&topic) {
                                            if !clients.contains(&client_id) {
                                                clients.push(client_id.clone());
                                            }
                                        } else {
                                            subs.insert(topic, vec![client_id.clone()]);
                                        }
                                        
                                        // 发送SUBACK响应
                                        if let Err(e) = send_suback(&mut socket, &buf[..n]).await {
                                            log::error!("Error sending SUBACK: {}", e);
                                        }
                                    }
                                },
                                _ => {
                                    log::warn!("Unhandled packet type: {}", packet_type);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading from socket: {}", e);
                        return Err(e.into());
                    }
                }
            }
            
            // 处理来自其他客户端的消息广播
            result = rx.recv() => {
                match result {
                    Ok((topic, message)) => {
                        // 检查此客户端是否订阅了该主题
                        let subs = subscriptions.lock().await;
                        if let Some(clients) = subs.get(&topic) {
                            if clients.contains(&client_id) {
                                // 发送消息给客户端
                                if let Err(e) = send_publish(&mut socket, &topic, &message).await {
                                    log::error!("Error sending publish message: {}", e);
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // 发送者关闭
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // 消息滞后
                        log::warn!("Warning: Lagged messages");
                    }
                }
            }
        }
        // 添加一个短暂的延迟，防止过于频繁的循环
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    Ok(())
}

/// 解析包类型
fn parse_packet_type(first_byte: u8) -> Option<u8> {
    Some(first_byte >> 4)
}

/// 处理CONNECT包
async fn handle_connect(socket: &mut TcpStream, _data: &[u8]) -> Result<String, Box<dyn Error>> {
    // 简化的CONNECT处理，实际实现应该解析完整的CONNECT包
    // 这里我们只发送一个CONNACK响应
    
    // CONNACK包: 固定头部(2字节) + 剩余长度(1字节) + 连接确认标志(1字节) + 返回码(1字节)
    let connack = [
        0x20, // CONNACK包类型
        0x02, // 剩余长度
        0x00, // 连接确认标志
        0x00, // 返回码: 0表示连接接受
    ];
    
    socket.write_all(&connack).await?;
    
    // 返回一个简单的客户端ID
    Ok("client-".to_string() + &rand::random::<u32>().to_string())
}

/// 解析PUBLISH包
fn parse_publish(data: &[u8]) -> Option<(String, String)> {
    if data.len() < 5 {
        return None;
    }
    
    // 解析剩余长度（简化处理，假设长度<128）
    let remaining_length = data[1] as usize;
    if data.len() < 2 + remaining_length {
        return None;
    }
    
    // 解析主题长度
    let topic_len = ((data[2] as u16) << 8 | (data[3] as u16)) as usize;
    
    // 检查缓冲区长度
    if data.len() < 4 + topic_len {
        return None;
    }
    
    // 解析主题
    let topic = String::from_utf8_lossy(&data[4..4 + topic_len]).to_string();
    
    // 解析消息内容
    let message_start = 4 + topic_len;
    if message_start >= data.len() {
        return None;
    }
    
    let message = String::from_utf8_lossy(&data[message_start..]).to_string();
    
    Some((topic, message))
}

/// 解析SUBSCRIBE包
fn parse_subscribe(data: &[u8]) -> Option<String> {
    if data.len() < 6 {
        return None;
    }
    
    // 解析剩余长度（简化处理，假设长度<128）
    let remaining_length = data[1] as usize;
    if data.len() < 2 + remaining_length {
        return None;
    }
    
    // 解析主题长度（跳过包标识符2字节）
    let topic_len = ((data[4] as u16) << 8 | (data[5] as u16)) as usize;
    
    // 检查缓冲区长度
    if data.len() < 6 + topic_len {
        return None;
    }
    
    // 解析主题
    let topic = String::from_utf8_lossy(&data[6..6 + topic_len]).to_string();
    
    Some(topic)
}

/// 发送PUBLISH包给客户端（简化版）
async fn send_publish(socket: &mut TcpStream, topic: &str, message: &str) -> Result<(), Box<dyn Error>> {
    // 构造一个简单的PUBLISH响应包
    let topic_bytes = topic.as_bytes();
    let message_bytes = message.as_bytes();
    
    // 固定头部 + 剩余长度 + 主题长度 + 主题 + 消息
    let mut response = Vec::new();
    response.push(0x30); // PUBLISH包类型 (PUBLISH)
    
    // 计算剩余长度（简化计算）
    let remaining_length = 2 + topic_bytes.len() + message_bytes.len();
    response.push(remaining_length as u8);
    
    // 主题长度（网络字节序）
    response.push((topic_bytes.len() >> 8) as u8);
    response.push(topic_bytes.len() as u8);
    
    // 主题
    response.extend_from_slice(topic_bytes);
    
    // 消息
    response.extend_from_slice(message_bytes);
    
    socket.write_all(&response).await?;
    Ok(())
}

/// 发送SUBACK包给客户端
async fn send_suback(socket: &mut TcpStream, subscribe_packet: &[u8]) -> Result<(), Box<dyn Error>> {
    // SUBACK包: 固定头部(1字节) + 剩余长度(1字节) + 包标识符(2字节) + 返回码(1字节)
    let mut response = Vec::new();
    
    // 固定头部 - SUBACK包类型
    response.push(0x90); // SUBACK包类型
    
    // 剩余长度 - 包标识符(2) + 返回码(1) = 3
    response.push(0x03);
    
    // 包标识符 - 从SUBSCRIBE包中复制
    if subscribe_packet.len() >= 4 {
        response.push(subscribe_packet[2]); // 包标识符高字节
        response.push(subscribe_packet[3]); // 包标识符低字节
    } else {
        response.push(0x00); // 默认包标识符高字节
        response.push(0x01); // 默认包标识符低字节
    }
    
    // 返回码 - 0x00表示QoS 0成功
    response.push(0x00);
    
    socket.write_all(&response).await?;
    Ok(())
}

/// 启动MQTT代理
pub async fn run_broker(addr: &str) -> Result<(), Box<dyn Error>> {
    let mut broker = MqttBroker::new(addr).await?;
    broker.run().await
}