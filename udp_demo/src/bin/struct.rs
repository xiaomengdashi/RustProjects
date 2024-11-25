use std::net::UdpSocket;
use std::io::{self, Error, ErrorKind};
use std::ffi::CStr;
use std::str;

#[repr(C)]
#[derive(Debug)]
struct MyStruct {
    id: i32,
    value: f32,
    len: u32,
    name: [u8; 20], // Rust的字符串类型不能直接与C字符串互操作，因此使用固定大小的数组
}

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:12345")?;

    let mut buf = [0; std::mem::size_of::<MyStruct>()];
    let (amt, _src) = socket.recv_from(&mut buf)?;

    if amt != std::mem::size_of::<MyStruct>() {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid data size"));
    }

    // 将接收到的数据解析为MyStruct
    let data = unsafe {
        std::ptr::read_unaligned(buf.as_ptr() as *const MyStruct)
    };

    // 打印结构体字段
    println!("Received struct: {:?}", data);

    // 将name字段转换为字符串
    let name = match CStr::from_bytes_until_nul(&data.name) {
        Ok(cstr) => str::from_utf8(cstr.to_bytes()).map_err(|e| Error::new(ErrorKind::InvalidData, e))?,
        Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Failed to parse C string")),
    };

    println!("Name: {}", name);

    Ok(())
}