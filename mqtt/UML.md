# RAN项目UML类图

## 类图

```mermaid
classDiagram
    class MqttClient {
        +client_id: String
        -stream: Option~TcpStream~
        -subscriptions: Arc~Mutex~HashMap~String, Vec~MessageCallback~~~~
        -state: Box~dyn State + Send~
        +new(client_id: String) MqttClient
        +connect(addr: &str) Result~(), Error~
        +subscribe(topic: String) Result~(), Error~
        +publish(topic: String, message: String) Result~(), Error~
        +disconnect() Result~(), Error~
        +on_message(topic: String, callback: F) void
        +start_listening() Result~(), Error~
        +get_state() ClientState
        +execute_command(command: &dyn Command) Result~(), Error~
        -set_state(state: Box~dyn State + Send~) void
        -transition_to(target_state: ClientState) void
        -create_subscribe_packet(topic: &str) Vec~u8~
        -create_publish_packet(topic: &str, message: &str) Vec~u8~
        -create_disconnect_packet() Vec~u8~
        -parse_packet(data: &[u8]) Option~(u8, Vec~u8~)~
        -extract_publish_payload(payload: &[u8]) Option~(String, String)~
    }
    
    class MqttBroker {
        -listener: TcpListener
        -subscriptions: Subscriptions
        -tx: broadcast::Sender~(String, String)~
        +new(addr: &str) Result~MqttBroker, Error~
        +run() Result~(), Error~
    }
    
    class ConnectPacket {
        +protocol_name: String
        +protocol_version: u8
        +connect_flags: u8
        +keep_alive: u16
        +client_id: String
        +username: Option~String~
        +password: Option~String~
        +new(client_id: String) ConnectPacket
        +encode() BytesMut
        -calculate_remaining_length() usize
        -encode_remaining_length(buffer: &mut BytesMut, length: usize) void
    }
    
    class PacketType {
        <<enumeration>>
        CONNECT
        CONNACK
        PUBLISH
        PUBACK
        PUBREC
        PUBREL
        PUBCOMP
        SUBSCRIBE
        SUBACK
        UNSUBSCRIBE
        UNSUBACK
        PINGREQ
        PINGRESP
        DISCONNECT
        +from_u8(value: u8) Option~PacketType~
    }
    
    class Main {
        +main() Result~(), Error~
        +start_server() Result~(), Error~
        +start_client() Result~(), Error~
    }
    
    class Command {
        <<interface>>
        +execute(client: &mut MqttClient) Result~(), Error~
        +get_name() &str
    }
    
    class ConnectCommand {
        +addr: String
        +new(addr: String) ConnectCommand
    }
    
    class PublishCommand {
        +topic: String
        +message: String
        +new(topic: String, message: String) PublishCommand
    }
    
    class SubscribeCommand {
        +topic: String
        +new(topic: String) SubscribeCommand
    }
    
    class DisconnectCommand {
        +new() DisconnectCommand
    }
    
    class State {
        <<interface>>
        +get_state() ClientState
        +can_execute_command(command_name: &str) bool
        +transition_to(target_state: ClientState) Option~Box~dyn State + Send~~
    }
    
    class DisconnectedState
    class ConnectingState
    class ConnectedState
    class DisconnectingState
    
    class ClientState {
        <<enumeration>>
        Disconnected
        Connecting
        Connected
        Disconnecting
    }
    
    Main --> MqttClient
    Main --> MqttBroker
    MqttClient --> ConnectPacket
    MqttClient --> PacketType
    MqttClient --> Command
    MqttClient --> State
    MqttClient --> ClientState
    MqttBroker --> PacketType
    ConnectPacket --> PacketType
    Command <|.. ConnectCommand
    Command <|.. PublishCommand
    Command <|.. SubscribeCommand
    Command <|.. DisconnectCommand
    State <|.. DisconnectedState
    State <|.. ConnectingState
    State <|.. ConnectedState
    State <|.. DisconnectingState
    State --> ClientState
    
    note right of MqttClient
        MQTT客户端实现：
        - 管理TCP连接
        - 实现发布/订阅功能
        - 处理消息回调
        - 持续监听服务器消息
        - 状态管理
        - 命令模式支持
    end note
    
    note right of MqttBroker
        MQTT服务端实现：
        - 监听客户端连接
        - 管理订阅信息
        - 广播消息给订阅者
    end note
    
    note right of ConnectPacket
        CONNECT数据包：
        - 封装连接参数
        - 实现编码逻辑
        - 计算包长度
    end note
    
    note right of Command
        命令模式接口：
        - 定义MQTT操作的通用接口
        - 支持异步执行
    end note
    
    note right of State
        状态模式接口：
        - 定义客户端状态行为
        - 支持状态转换
    end note
```

## 组件关系说明

1. **Main类**：程序入口点，负责根据命令行参数启动客户端或服务端
2. **MqttClient类**：MQTT客户端实现，负责连接、发布、订阅等操作
3. **MqttBroker类**：MQTT服务端实现，负责监听连接、管理订阅、分发消息
4. **ConnectPacket类**：MQTT CONNECT包的实现，用于建立客户端连接
5. **PacketType枚举**：定义MQTT协议中的各种包类型
6. **Command接口及实现类**：命令模式实现，封装MQTT操作
7. **State接口及实现类**：状态模式实现，管理客户端状态
8. **ClientState枚举**：定义客户端的所有可能状态

## 设计模式应用

- **工厂模式**：`ConnectPacket::new()`方法用于创建CONNECT包实例
- **观察者模式**：`MqttClient::on_message()`方法用于注册消息回调
- **策略模式**：`PacketType`枚举用于区分不同类型的包处理策略
- **并发模式**：使用Tokio的异步运行时和同步原语处理并发连接
- **命令模式**：将MQTT操作封装为命令对象，支持操作队列和扩展
- **状态模式**：管理MQTT客户端状态，根据状态限制可执行的操作