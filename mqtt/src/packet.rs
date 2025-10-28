/// MQTT数据包结构和处理
use bytes::{BytesMut, BufMut};
use crate::protocol::*;

#[derive(Debug)]
pub struct ConnectPacket {
    pub protocol_name: String,
    pub protocol_version: u8,
    pub connect_flags: u8,
    pub keep_alive: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl ConnectPacket {
    pub fn new(client_id: String) -> Self {
        ConnectPacket {
            protocol_name: MQTT_PROTOCOL_NAME.to_string(),
            protocol_version: MQTT_PROTOCOL_VERSION,
            connect_flags: CLEAN_SESSION, // 默认只设置CLEAN_SESSION
            keep_alive: 60, // 默认60秒
            client_id,
            username: None,
            password: None,
        }
    }

    /// 将ConnectPacket编码为字节流
    pub fn encode(&self) -> BytesMut {
        let mut buffer = BytesMut::new();
        
        // 固定头部 - 控制包类型和标志
        let header = (PacketType::CONNECT as u8) << 4;
        buffer.put_u8(header);
        
        // 计算剩余长度（可变头部+载荷）
        let remaining_length = self.calculate_remaining_length();
        self.encode_remaining_length(&mut buffer, remaining_length);
        
        // 可变头部
        // 协议名长度和协议名
        buffer.put_u16(self.protocol_name.len() as u16);
        buffer.extend_from_slice(self.protocol_name.as_bytes());
        
        // 协议版本
        buffer.put_u8(self.protocol_version);
        
        // 连接标志
        buffer.put_u8(self.connect_flags);
        
        // 保持连接时间
        buffer.put_u16(self.keep_alive);
        
        // 载荷 - 客户端标识符
        buffer.put_u16(self.client_id.len() as u16);
        buffer.extend_from_slice(self.client_id.as_bytes());
        
        // 用户名和密码（如果存在）
        if let Some(ref username) = self.username {
            buffer.put_u16(username.len() as u16);
            buffer.extend_from_slice(username.as_bytes());
        }
        
        if let Some(ref password) = self.password {
            buffer.put_u16(password.len() as u16);
            buffer.extend_from_slice(password.as_bytes());
        }
        
        buffer
    }
    
    /// 计算剩余长度
    fn calculate_remaining_length(&self) -> usize {
        // 协议名长度(2字节) + 协议名 + 协议版本(1字节) + 连接标志(1字节) + 
        // 保持连接时间(2字节) + 客户端ID长度(2字节) + 客户端ID +
        // 用户名长度(2字节) + 用户名(如果有) +
        // 密码长度(2字节) + 密码(如果有)
        
        let mut length = 2 + self.protocol_name.len() + 1 + 1 + 2 + 2 + self.client_id.len();
        
        if let Some(ref username) = self.username {
            length += 2 + username.len();
        }
        
        if let Some(ref password) = self.password {
            length += 2 + password.len();
        }
        
        length
    }
    
    /// 编码剩余长度（使用可变长度编码）
    fn encode_remaining_length(&self, buffer: &mut BytesMut, length: usize) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_packet_encoding() {
        let packet = ConnectPacket::new("test-client".to_string());
        let encoded = packet.encode();
        
        // 验证基本结构
        assert!(!encoded.is_empty());
        // 第一个字节应该是CONNECT包类型
        assert_eq!(encoded[0], (PacketType::CONNECT as u8) << 4);
    }
    
    #[test]
    fn test_connect_packet_with_username() {
        let mut packet = ConnectPacket::new("test-client".to_string());
        packet.username = Some("testuser".to_string());
        let encoded = packet.encode();
        
        // 验证基本结构
        assert!(!encoded.is_empty());
        // 第一个字节应该是CONNECT包类型
        assert_eq!(encoded[0], (PacketType::CONNECT as u8) << 4);
    }
    
    #[test]
    fn test_connect_packet_with_password() {
        let mut packet = ConnectPacket::new("test-client".to_string());
        packet.username = Some("testuser".to_string());
        packet.password = Some("testpass".to_string());
        let encoded = packet.encode();
        
        // 验证基本结构
        assert!(!encoded.is_empty());
        // 第一个字节应该是CONNECT包类型
        assert_eq!(encoded[0], (PacketType::CONNECT as u8) << 4);
    }
    
    #[test]
    fn test_encode_remaining_length() {
        let packet = ConnectPacket::new("test-client".to_string());
        
        // 测试小长度值
        let mut buffer = BytesMut::new();
        packet.encode_remaining_length(&mut buffer, 127);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0], 127);
        
        // 测试需要多个字节的长度值
        let mut buffer = BytesMut::new();
        packet.encode_remaining_length(&mut buffer, 128);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], 128 | 128); // 继续位 + 0
        assert_eq!(buffer[1], 1); // 1
        
        // 测试更大的长度值
        let mut buffer = BytesMut::new();
        packet.encode_remaining_length(&mut buffer, 16383);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], 128 | 127); // 继续位 + 127
        assert_eq!(buffer[1], 127); // 127
    }
    
    #[test]
    fn test_calculate_remaining_length() {
        let packet = ConnectPacket::new("test-client".to_string());
        let length = packet.calculate_remaining_length();
        
        // 验证长度计算是否合理
        assert!(length > 0);
        // 对于基本的client-id，应该包含协议名、版本、标志、keep_alive、client_id等字段
        assert!(length > 20);
    }
}