use std::net::UdpSocket;
use std::io::{self, Error, ErrorKind};
use std::mem;
use std::slice;
use std::ffi::CStr;

#[repr(C)]
#[derive(Debug)]
struct SubMessage {
    type_: u32,
    age: u32,
}

#[repr(C)]
#[derive(Debug)]
struct Message {
    id: i32,
    value: f32,
    len: i32,
    name: [u8; 20],
    msg_len: i32,
}

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:12345")?;

    let mut buf = [0; 1024]; // Adjust buffer size as needed
    let (amt, _src) = socket.recv_from(&mut buf)?;

    println!("Received {} bytes", amt);
    println!("Expected message header size: {}", mem::size_of::<Message>());

    if amt < mem::size_of::<Message>() {
        return Err(Error::new(ErrorKind::InvalidData, "Not enough data"));
    }

    // 将接收到的数据解析为Message头部
    let msg_header = unsafe {
        &*(buf.as_ptr() as *const Message)
    };

    println!("Received Message Header: {:?}", msg_header);

    // 检查是否包含足够的子消息数据
    let submsg_size = msg_header.msg_len as usize * mem::size_of::<SubMessage>();
    println!("Expected submessage size: {}", submsg_size);
    if amt < mem::size_of::<Message>() + submsg_size {
        return Err(Error::new(ErrorKind::InvalidData, "Not enough data for submessages"));
    }

    // 将子消息数据解析为SubMessage数组
    let submsg_slice = unsafe {
        slice::from_raw_parts((buf.as_ptr() as *const u8).add(mem::size_of::<Message>()) as *const SubMessage, msg_header.msg_len as usize)
    };

    // 将name字段转换为字符串
    let name = match CStr::from_bytes_until_nul(&msg_header.name) {
        Ok(cstr) => cstr.to_str().map_err(|e| Error::new(ErrorKind::InvalidData, e))?,
        Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Failed to parse C string")),
    };
    println!("Name: {}", name);

    // 打印子消息
    for submsg in submsg_slice {
        println!("SubMessage: {:?}", submsg);
    }

    Ok(())
}