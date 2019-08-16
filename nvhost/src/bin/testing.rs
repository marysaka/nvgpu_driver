use nvhost::*;

pub fn main() {
    let _nvhost_ctrl = NvHostCtrl::new().unwrap();
    println!("Hello World");
}
