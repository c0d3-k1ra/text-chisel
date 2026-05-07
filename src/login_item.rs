use std::path::PathBuf;

use anyhow::Context;

const LABEL: &str = "com.textchisel.app";

fn plist_path() -> anyhow::Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;
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
    let exe = std::env::current_exe().context("failed to get current executable path")?;
    let p = plist_path()?;

    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).context("failed to create LaunchAgents directory")?;
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

    std::fs::write(&p, &plist).context("failed to write LaunchAgent plist")?;

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
        std::fs::remove_file(&p).context("failed to remove LaunchAgent plist")?;
    }

    log::info!("launch at login disabled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- xml_escape ---

    #[test]
    fn xml_escape_ampersand() {
        assert_eq!(xml_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn xml_escape_angle_brackets() {
        assert_eq!(xml_escape("a < b > c"), "a &lt; b &gt; c");
    }

    #[test]
    fn xml_escape_double_quote() {
        assert_eq!(xml_escape(r#"say "hello""#), "say &quot;hello&quot;");
    }

    #[test]
    fn xml_escape_single_quote() {
        assert_eq!(xml_escape("it's fine"), "it&apos;s fine");
    }

    #[test]
    fn xml_escape_plain_string_unchanged() {
        assert_eq!(
            xml_escape("/Applications/Text Chisel.app"),
            "/Applications/Text Chisel.app"
        );
    }

    #[test]
    fn xml_escape_multiple_special_chars() {
        assert_eq!(xml_escape("a & <b>"), "a &amp; &lt;b&gt;");
    }

    // --- plist_path ---

    #[test]
    fn plist_path_ends_with_expected_filename() {
        let p = plist_path().unwrap();
        assert!(p.ends_with("Library/LaunchAgents/com.textchisel.app.plist"));
    }

    // --- roundtrip (manual) ---

    #[test]
    #[ignore = "Writes and removes ~/Library/LaunchAgents/com.textchisel.app.plist on the real filesystem. Run manually to verify enable/disable works correctly."]
    fn test_launch_at_login_roundtrip() {
        // Start clean.
        if is_enabled() {
            disable().expect("failed to disable before test");
        }
        assert!(!is_enabled(), "should be disabled before enable");

        enable().expect("enable failed");
        assert!(is_enabled(), "should be enabled after enable");

        // Verify plist is well-formed and contains expected fields.
        let p = plist_path().unwrap();
        let content = std::fs::read_to_string(&p).expect("plist not found after enable");
        assert!(
            content.contains("com.textchisel.app"),
            "plist missing Label"
        );
        assert!(content.contains("RunAtLoad"), "plist missing RunAtLoad");
        let exe = std::env::current_exe().unwrap();
        assert!(
            content.contains(&exe.to_string_lossy().replace('&', "&amp;")),
            "plist missing executable path"
        );
        println!("Plist at {}:\n{}", p.display(), content);

        disable().expect("disable failed");
        assert!(!is_enabled(), "should be disabled after disable");
        assert!(!p.exists(), "plist file should be removed after disable");
    }
}
