import aiohttp
import asyncio
import datetime
import signal
import sys
import random

class URLMonitor:
    def __init__(self, base_url="http://127.0.0.1:8080"):
        self.base_url = base_url
        self.running = True
        self.routes = ["ping", "status"]
        self.session = None
        signal.signal(signal.SIGINT, self.signal_handler)
        
    def signal_handler(self, signum, frame):
        print("\n正在停止监控...")
        self.running = False

    async def init_session(self):
        """初始化 aiohttp session"""
        if self.session is None:
            timeout = aiohttp.ClientTimeout(total=5)  # 5秒总超时
            self.session = aiohttp.ClientSession(timeout=timeout)

    async def cleanup(self):
        """清理资源"""
        if self.session:
            await self.session.close()

    async def send_request(self, route):
        """发送单个请求"""
        url = f"{self.base_url}/{route}"
        timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        try:
            async with self.session.get(url) as response:
                print(f"[{timestamp}] 请求: {route}", end=', ')
                print(f"状态码: {response.status}", end=', ')
                try:
                    result = await response.json()
                    print(f"响应: {result}")
                except:
                    text = await response.text()
                    print(f"响应: {text}")
                    
        except asyncio.TimeoutError:
            print(f"[{timestamp}] 错误: {route} 请求超时")
        except aiohttp.ClientError as e:
            print(f"[{timestamp}] 错误: {route} - {str(e)}")
        except Exception as e:
            print(f"[{timestamp}] 未知错误: {route} - {str(e)}")

    async def random_monitor(self):
        """随机发送请求"""
        await self.init_session()
        
        try:
            while self.running:
                route = random.choice(self.routes)
                # 使用 asyncio.wait_for 添加额外的超时保护
                try:
                    await asyncio.wait_for(
                        self.send_request(route),
                        timeout=6  # 6秒超时（比 session 超时稍长）
                    )
                except asyncio.TimeoutError:
                    timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                    print(f"[{timestamp}] 错误: {route} 请求执行超时")
                
                # 随机等待0.5到2秒
                await asyncio.sleep(random.uniform(0.5, 2))
        finally:
            await self.cleanup()

    async def start_monitoring(self):
        print(f"开始监控 {self.base_url}")
        print("按 Ctrl+C 停止监控")
        
        try:
            await self.random_monitor()
        except Exception as e:
            print(f"监控发生错误: {e}")
        finally:
            await self.cleanup()
            print("\n监控已停止")

def main():
    monitor = URLMonitor()
    try:
        if sys.platform == 'win32':
            asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
        asyncio.run(monitor.start_monitoring())
    except KeyboardInterrupt:
        print("\n程序已退出")

if __name__ == "__main__":
    main()