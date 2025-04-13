fn main() {
    println!("cargo::rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo::rustc-link-arg=-ObjC");
}