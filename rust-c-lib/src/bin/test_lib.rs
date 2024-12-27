use std::os::raw::c_double;
use std::os::raw::c_int; // 32位 // 64位

mod mytime;
use mytime::*;
use std::ffi::CStr;

// 从标准库 libc 中引入三个函数。
// 此处是 Rust 对三个 C 函数的声明：
extern "C" {
    fn abs(num: c_int) -> c_int;
    fn sqrt(num: c_double) -> c_double;
    fn pow(num: c_double, power: c_double) -> c_double;
}

fn main() {
    let x: i32 = -123;
    println!("\n{x}的绝对值是: {}.", unsafe { abs(x) });
    let n: f64 = 9.0;
    let p: f64 = 3.0;
    println!("\n{n}的{p}次方是: {}.", unsafe { pow(n, p) });
    let mut y: f64 = 64.0;
    println!("\n{y}的平方根是: {}.", unsafe { sqrt(y) });
    y = -3.14;
    println!("\n{y}的平方根是: {}.", unsafe { sqrt(y) }); //** NaN = NotaNumber（不是数字）

    let mut sometime = StructTM {
        tm_year: 1,
        tm_mon: 1,
        tm_mday: 1,
        tm_hour: 1,
        tm_min: 1,
        tm_sec: 1,
        tm_isdst: -1,
        tm_wday: 1,
        tm_yday: 1,
    };

    
    unsafe {
        let c_ptr = &mut sometime; // 裸指针
                                   // 调用，转化，并拥有
                                   // 返回的 C 字符串
        let char_ptr = asctime(c_ptr);
        let c_str = CStr::from_ptr(char_ptr);
        println!("{:#?}", c_str.to_str());

        
        let utc = mktime(&mut sometime);
        println!("{}", utc);
    }
}