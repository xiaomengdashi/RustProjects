/// MQTT协议常量和类型定义
pub const MQTT_PROTOCOL_NAME: &str = "MQTT";
pub const MQTT_PROTOCOL_VERSION: u8 = 4; // MQTT 3.1.1

// 连接标志位
pub const USERNAME_FLAG: u8 = 0b10000000;
pub const PASSWORD_FLAG: u8 = 0b01000000;
pub const WILL_RETAIN: u8 = 0b00100000;
pub const WILL_QOS_MASK: u8 = 0b00011000;
pub const WILL_FLAG: u8 = 0b00000100;
pub const CLEAN_SESSION: u8 = 0b00000010;

// MQTT控制包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    CONNECT = 1,
    CONNACK = 2,
    PUBLISH = 3,
    PUBACK = 4,
    PUBREC = 5,
    PUBREL = 6,
    PUBCOMP = 7,
    SUBSCRIBE = 8,
    SUBACK = 9,
    UNSUBSCRIBE = 10,
    UNSUBACK = 11,
    PINGREQ = 12,
    PINGRESP = 13,
    DISCONNECT = 14,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<PacketType> {
        match value {
            1 => Some(PacketType::CONNECT),
            2 => Some(PacketType::CONNACK),
            3 => Some(PacketType::PUBLISH),
            4 => Some(PacketType::PUBACK),
            5 => Some(PacketType::PUBREC),
            6 => Some(PacketType::PUBREL),
            7 => Some(PacketType::PUBCOMP),
            8 => Some(PacketType::SUBSCRIBE),
            9 => Some(PacketType::SUBACK),
            10 => Some(PacketType::UNSUBSCRIBE),
            11 => Some(PacketType::UNSUBACK),
            12 => Some(PacketType::PINGREQ),
            13 => Some(PacketType::PINGRESP),
            14 => Some(PacketType::DISCONNECT),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_type_from_u8() {
        assert_eq!(PacketType::from_u8(1), Some(PacketType::CONNECT));
        assert_eq!(PacketType::from_u8(2), Some(PacketType::CONNACK));
        assert_eq!(PacketType::from_u8(3), Some(PacketType::PUBLISH));
        assert_eq!(PacketType::from_u8(8), Some(PacketType::SUBSCRIBE));
        assert_eq!(PacketType::from_u8(9), Some(PacketType::SUBACK));
        assert_eq!(PacketType::from_u8(14), Some(PacketType::DISCONNECT));
        assert_eq!(PacketType::from_u8(99), None);
    }
    
    #[test]
    fn test_protocol_constants() {
        assert_eq!(MQTT_PROTOCOL_NAME, "MQTT");
        assert_eq!(MQTT_PROTOCOL_VERSION, 4);
        assert_eq!(CLEAN_SESSION, 0b00000010);
        assert_eq!(USERNAME_FLAG, 0b10000000);
        assert_eq!(PASSWORD_FLAG, 0b01000000);
    }
}