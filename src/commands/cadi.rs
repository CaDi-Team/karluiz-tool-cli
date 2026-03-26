//! The `ktool cadi` hero screen — retro 8-bit karluiz branding.

/// Prints the karluiz hero screen to stdout.
pub fn run() {
    println!(
        r#" ╔═══════════════════════════════════════════════════════╗
 ║                                                       ║
 ║   ██╗  ██╗ █████╗ ██████╗ ██╗     ██╗   ██╗██╗███╗   ║
 ║   ██║ ██╔╝██╔══██╗██╔══██╗██║     ██║   ██║██║╚══╝   ║
 ║   █████╔╝ ███████║██████╔╝██║     ██║   ██║██║███╗    ║
 ║   ██╔═██╗ ██╔══██║██╔══██╗██║     ██║   ██║██║╚══╝   ║
 ║   ██║  ██╗██║  ██║██║  ██║███████╗╚██████╔╝██║███╗   ║
 ║   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══╝   ║
 ║                                                       ║
 ║              DEVELOPER BY PASSION                     ║
 ║            COMMODORE 64 FOREVER                       ║
 ║                                                       ║
 ║    v{version} · Made by CaDi Labs with love <3        ║
 ║                                                       ║
 ╚═══════════════════════════════════════════════════════╝"#,
        version = env!("CARGO_PKG_VERSION")
    );
}
