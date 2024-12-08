import socket
import json
import time
from datetime import datetime

class UDPServer:
    def __init__(self, host='127.0.0.1', port=8889):
        self.host = host
        self.port = port
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        
    def start(self):
        try:
            self.sock.bind((self.host, self.port))
            print(f"UDP服务器启动在 {self.host}:{self.port}")
            
            while True:
                try:
                    # 接收数据
                    data, addr = self.sock.recvfrom(1024)
                    self.handle_request(data, addr)
                except Exception as e:
                    print(f"处理请求时出错: {e}")
                    time.sleep(1)
                    
        except Exception as e:
            print(f"服务器错误: {e}")
        finally:
            self.sock.close()
            
    def handle_request(self, data, addr):
        try:
            # 解析接收到的JSON数据
            message = json.loads(data.decode('utf-8'))
            route = message['header']['route']
            request_id = message['header']['request_id']
            
            print(f"\n收到请求:")
            print(f"来源: {addr}")
            print(f"路由: {route}")
            print(f"请求ID: {request_id}")
            
            # 首先发送 ACK 响应
            ack_message = {
                "header": {
                    "msg_type": "ack",
                    "route": route,
                    "request_id": request_id
                },
                "body": ""
            }
            self.sock.sendto(json.dumps(ack_message).encode('utf-8'), addr)
            print(f"已发送 ACK (ID: {request_id})")
            
            # 生成并发送完整响应
            response_body = self.generate_response(route)
            complete_message = {
                "header": {
                    "msg_type": "complete",
                    "route": route,
                    "request_id": request_id
                },
                "body": response_body
            }
            
            # 短暂延迟以模拟处理时间
            time.sleep(0.1)
            
            # 发送响应
            self.sock.sendto(json.dumps(complete_message).encode('utf-8'), addr)
            print(f"已发送完整响应 (ID: {request_id}): {json.dumps(complete_message)}\n")
            
        except json.JSONDecodeError as e:
            print(f"JSON解析错误: {e}")
            error_message = {
                "header": {
                    "msg_type": "error",
                    "route": "error",
                    "request_id": "error"
                },
                "body": "Invalid JSON format"
            }
            self.sock.sendto(json.dumps(error_message).encode('utf-8'), addr)
        except Exception as e:
            print(f"处理请求时出错: {e}")
            error_message = {
                "header": {
                    "msg_type": "error",
                    "route": "error",
                    "request_id": message.get('header', {}).get('request_id', 'error')
                },
                "body": str(e)
            }
            self.sock.sendto(json.dumps(error_message).encode('utf-8'), addr)
            
    def generate_response(self, route):
        """根据不同的路由生成不同的响应"""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        # 根据路由生成响应内容
        if route == 'ping':
            return "pong"
        elif route == 'time':
            return timestamp
        elif route == 'echo':
            return f"Echo: {route}"
        elif route == 'status':
            return "Server is running"
        else:
            return f"Unknown route: {route}"

def main():
    server = UDPServer()
    try:
        server.start()
    except KeyboardInterrupt:
        print("\n服务器已停止")

if __name__ == "__main__":
    main()