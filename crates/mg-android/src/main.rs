#[cfg(not(target_os = "android"))]
fn main() {
    mg_android::run_desktop();
}

#[cfg(target_os = "android")]
fn main() {}
