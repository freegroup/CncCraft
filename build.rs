// the translations in locales/ are compiled into the program (rust-i18n): rebuild when they change
fn main() {
    println!("cargo:rerun-if-changed=locales");
}
