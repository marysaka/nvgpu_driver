use nvhost::*;
use nvmap::*;

pub fn main() {
    let nvhost_ctrl = NvHostCtrl::new().unwrap();
    println!("Hello World");
}
