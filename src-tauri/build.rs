fn main() {
    println!("cargo:rustc-link-search=native=lib"); // lib 是存放 mylib.lib 的目录

    tauri_build::build()
}
