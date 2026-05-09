use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let icon_path = workspace_dir.join("resource").join("icon.ico");

    if icon_path.exists() {
        let mut res = winres::WindowsResource::new();
        res.set_icon(icon_path.to_str().unwrap());
        res.set("ProductName", "Xime");
        res.set("FileDescription", "Xime 五笔输入法设置");
        res.compile().unwrap();
    }

    println!("cargo:rerun-if-changed={}", icon_path.display());
}