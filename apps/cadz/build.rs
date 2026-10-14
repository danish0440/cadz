//! Windows only: embed the app icon and version info into `cadz.exe`. Elsewhere a no-op.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/cadz.ico");
    println!("cargo:rerun-if-env-changed=CADZ_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=CADZ_BUILD_SHA");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    if std::path::Path::new("../../assets/app-icon/cadz.ico").exists() {
        res.set_icon("../../assets/app-icon/cadz.ico");
    }
    res.set("ProductName", "CadZ")
        .set("FileDescription", "CadZ computer-aided design")
        .set("CompanyName", "Learning Machines LLC")
        .set("LegalCopyright", "Copyright (c) 2026 ArtCraft Team and the CADCraft contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "cadz.exe")
        .set("InternalName", "cadz");
    if let Err(e) = res.compile() {
        if std::env::var_os("CADZ_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=cadz.exe built without icon/version resources: {e}");
    }
}
