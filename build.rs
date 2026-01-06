// build.rs
//use winres::WindowsResource;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/assets/magnifier.ico"); // Ide kell egy .ico fájl!
        res.compile().unwrap();
    }
}
