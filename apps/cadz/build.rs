//! Windows only: embed the app icon and version info into `cadz.exe`. Elsewhere a no-op.
//!
//! The build channel comes from `CADZ_BUILD_CHANNEL`: `nightly-v1` compiles a Nightly build
//! with its own icon and version, `stable`/unset is the stable build, and anything else is refused
//! rather than silently producing a stable binary with a nightly name.

#[path = "src/build_channel.rs"]
mod build_channel;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/build_channel.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/cadz.ico");
    println!("cargo:rerun-if-changed=../../assets/app-icon/cadz-nightly.ico");
    println!("cargo:rerun-if-env-changed=CADZ_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=CADZ_BUILD_SHA");
    println!("cargo:rerun-if-env-changed=CADZ_BUILD_CHANNEL");
    println!("cargo:rerun-if-env-changed=CADZ_VERSION");
    println!("cargo:rustc-check-cfg=cfg(cadz_nightly_v1)");
    let nightly = match std::env::var("CADZ_BUILD_CHANNEL").as_deref() {
        Ok("nightly-v1") => {
            println!("cargo:rustc-cfg=cadz_nightly_v1");
            true
        }
        // An empty value (an unset CI input) means the default stable build.
        Ok("stable" | "") | Err(std::env::VarError::NotPresent) => false,
        _ => {
            eprintln!("unsupported CADZ_BUILD_CHANNEL (expected stable or nightly-v1)");
            std::process::exit(1);
        }
    };
    // The version the build reports: the prepared feed identity for a nightly (refused when it is
    // missing or is not a nightly identity), the crate version otherwise.
    let built_version = if nightly {
        match std::env::var("CADZ_VERSION") {
            Ok(version) if build_channel::is_nightly_version(&version) => version,
            _ => {
                eprintln!("nightly-v1 requires an explicit CADZ_VERSION feed identity");
                std::process::exit(1);
            }
        }
    } else {
        std::env::var("CARGO_PKG_VERSION").unwrap_or_default()
    };
    println!("cargo:rustc-env=CADZ_BUILT_VERSION={built_version}");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon(build_channel::windows_icon_path(nightly))
        .set("ProductName", if nightly { "CadZ Nightly" } else { "CadZ" })
        .set("FileDescription", if nightly { "CadZ Nightly computer-aided design" } else { "CadZ computer-aided design" })
        .set("CompanyName", "Learning Machines LLC")
        .set("LegalCopyright", "Copyright (c) 2026 ArtCraft Team and the CADCraft contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "cadz.exe")
        .set("InternalName", "cadz");
    if nightly {
        // The Launcher verifies the PE ProductVersion against the exact feed identity, not FileVersion.
        res.set("ProductVersion", &built_version);
    }
    if let Err(e) = res.compile() {
        if std::env::var_os("CADZ_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=cadz.exe built without icon/version resources: {e}");
    }
}
