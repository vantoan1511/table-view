use std::env;
use std::path::Path;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let icon_path = Path::new(&manifest_dir).join("../../public/favicon.ico");

        let mut res = winres::WindowsResource::new();
        res.set_version_info(winres::VersionInfo::FILEVERSION, 0x0000000600030000);
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0000000600030000);
        if icon_path.exists() {
            res.set_icon(icon_path.to_str().unwrap());
        }
        res.set("ProductName", "Table View Database Manager");
        res.set("FileDescription", "Table View Native Backend Extension");
        res.set("CompanyName", "Toan Nguyen");
        res.set("LegalCopyright", "Copyright (c) 2026 Table View. All rights reserved.");
        res.set("InternalName", "db-bridge.exe");
        res.set("OriginalFilename", "db-bridge.exe");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }
}
