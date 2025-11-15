#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        // Only set manifest without touching version resources
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
        res.compile().unwrap();
    }

    tauri_build::build()
}
