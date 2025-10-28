/// 状态模式实现模块
/// 管理MQTT客户端的不同状态

/// 客户端状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

/// 状态 trait，定义了状态相关的行为
pub trait State {
    fn get_state(&self) -> ClientState;
    fn can_execute_command(&self, command_name: &str) -> bool;
    fn transition_to(&self, target_state: ClientState) -> Option<Box<dyn State + Send>>;
}

/// 断开连接状态
pub struct DisconnectedState;

impl State for DisconnectedState {
    fn get_state(&self) -> ClientState {
        ClientState::Disconnected
    }
    
    fn can_execute_command(&self, command_name: &str) -> bool {
        command_name == "Connect"
    }
    
    fn transition_to(&self, target_state: ClientState) -> Option<Box<dyn State + Send>> {
        match target_state {
            ClientState::Connecting => Some(Box::new(ConnectingState)),
            _ => None,
        }
    }
}

/// 连接中状态
pub struct ConnectingState;

impl State for ConnectingState {
    fn get_state(&self) -> ClientState {
        ClientState::Connecting
    }
    
    fn can_execute_command(&self, _command_name: &str) -> bool {
        false // 连接中状态下不能执行任何命令
    }
    
    fn transition_to(&self, target_state: ClientState) -> Option<Box<dyn State + Send>> {
        match target_state {
            ClientState::Connected => Some(Box::new(ConnectedState)),
            ClientState::Disconnected => Some(Box::new(DisconnectedState)),
            _ => None,
        }
    }
}

/// 已连接状态
pub struct ConnectedState;

impl State for ConnectedState {
    fn get_state(&self) -> ClientState {
        ClientState::Connected
    }
    
    fn can_execute_command(&self, command_name: &str) -> bool {
        command_name == "Publish" || command_name == "Subscribe" || command_name == "Disconnect"
    }
    
    fn transition_to(&self, target_state: ClientState) -> Option<Box<dyn State + Send>> {
        match target_state {
            ClientState::Disconnecting => Some(Box::new(DisconnectingState)),
            ClientState::Disconnected => Some(Box::new(DisconnectedState)),
            _ => None,
        }
    }
}

/// 断开连接中状态
pub struct DisconnectingState;

impl State for DisconnectingState {
    fn get_state(&self) -> ClientState {
        ClientState::Disconnecting
    }
    
    fn can_execute_command(&self, _command_name: &str) -> bool {
        false // 断开连接中状态下不能执行任何命令
    }
    
    fn transition_to(&self, target_state: ClientState) -> Option<Box<dyn State + Send>> {
        match target_state {
            ClientState::Disconnected => Some(Box::new(DisconnectedState)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_state_enum() {
        assert_eq!(ClientState::Disconnected, ClientState::Disconnected);
        assert_ne!(ClientState::Connected, ClientState::Disconnected);
    }
    
    #[test]
    fn test_state_transitions() {
        let disconnected = DisconnectedState;
        let connected = ConnectedState;
        
        assert_eq!(disconnected.get_state(), ClientState::Disconnected);
        assert_eq!(connected.get_state(), ClientState::Connected);
    }
    
    #[test]
    fn test_command_permissions() {
        let disconnected = DisconnectedState;
        let connected = ConnectedState;
        
        // 断开连接状态下只能执行连接命令
        assert!(disconnected.can_execute_command("Connect"));
        assert!(!disconnected.can_execute_command("Publish"));
        assert!(!disconnected.can_execute_command("Subscribe"));
        
        // 已连接状态下可以执行发布和订阅命令
        assert!(connected.can_execute_command("Publish"));
        assert!(connected.can_execute_command("Subscribe"));
        assert!(!connected.can_execute_command("Connect")); // 已连接时不能再次连接
    }
    
    #[test]
    fn test_state_transitions_logic() {
        let disconnected = DisconnectedState;
        let connecting = ConnectingState;
        let connected = ConnectedState;
        let disconnecting = DisconnectingState;
        
        // 测试断开连接状态的转换
        assert!(disconnected.transition_to(ClientState::Connecting).is_some());
        assert!(disconnected.transition_to(ClientState::Connected).is_none());
        
        // 测试连接中状态的转换
        assert!(connecting.transition_to(ClientState::Connected).is_some());
        assert!(connecting.transition_to(ClientState::Disconnected).is_some());
        assert!(connecting.transition_to(ClientState::Disconnecting).is_none());
        
        // 测试已连接状态的转换
        assert!(connected.transition_to(ClientState::Disconnecting).is_some());
        assert!(connected.transition_to(ClientState::Disconnected).is_some());
        assert!(connected.transition_to(ClientState::Connecting).is_none());
        
        // 测试断开连接中状态的转换
        assert!(disconnecting.transition_to(ClientState::Disconnected).is_some());
        assert!(disconnecting.transition_to(ClientState::Connected).is_none());
    }
}