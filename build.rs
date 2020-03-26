// build.rs

extern crate winres;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico");
    println!("{:?}", res);
    res.compile().unwrap();
}
