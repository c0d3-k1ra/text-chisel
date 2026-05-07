use anyhow::Context;
use std::path::PathBuf;

const LABEL: &str = "com.textchisel.app";

fn plist_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;
    Ok(home
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", LABEL)))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(crate) fn is_enabled() -> bool {
    plist_path().map(|p| p.exists()).unwrap_or(false)
}

pub(crate) fn enable() -> anyhow::Result<()> {
    let exe = std::env::current_exe()
        .context("failed to get current executable path")?;
    let p = plist_path()?;

    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .context("failed to create LaunchAgents directory")?;
    }

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{label}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{exe}</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>KeepAlive</key>
	<false/>
</dict>
</plist>"#,
        label = LABEL,
        // XML-escaped: macOS paths can legally contain &, <, >, ", '
        exe = xml_escape(&exe.to_string_lossy())
    );

    std::fs::write(&p, &plist)
        .context("failed to write LaunchAgent plist")?;

    // Intentionally not calling `launchctl bootstrap` here — doing so with
    // RunAtLoad=true would spawn a second instance immediately. The plist
    // in ~/Library/LaunchAgents/ is sufficient for macOS to pick it up at
    // next login.
    log::info!("launch at login enabled ({})", p.display());
    Ok(())
}

pub(crate) fn disable() -> anyhow::Result<()> {
    let p = plist_path()?;

    if p.exists() {
        std::fs::remove_file(&p)
            .context("failed to remove LaunchAgent plist")?;
    }

    log::info!("launch at login disabled");
    Ok(())
}
