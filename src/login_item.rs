use std::path::{Path, PathBuf};

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

fn write_plist(p: &Path, exe: &Path) -> anyhow::Result<()> {
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

    std::fs::write(p, &plist).context("failed to write LaunchAgent plist")
}

fn remove_plist(p: &Path) -> anyhow::Result<()> {
    if p.exists() {
        std::fs::remove_file(p).context("failed to remove LaunchAgent plist")?;
    }
    Ok(())
}

pub(crate) fn is_enabled() -> bool {
    plist_path().map(|p| p.exists()).unwrap_or(false)
}

pub(crate) fn enable() -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("failed to get current executable path")?;
    let p = plist_path()?;
    write_plist(&p, &exe)?;
    // Intentionally not calling `launchctl bootstrap` here — doing so with
    // RunAtLoad=true would spawn a second instance immediately. The plist
    // in ~/Library/LaunchAgents/ is sufficient for macOS to pick it up at
    // next login.
    log::info!("launch at login enabled ({})", p.display());
    Ok(())
}

pub(crate) fn disable() -> anyhow::Result<()> {
    let p = plist_path()?;
    remove_plist(&p)?;
    log::info!("launch at login disabled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_plist(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("text-chisel-test-{}.plist", tag))
    }

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

    // --- write_plist ---

    #[test]
    fn write_plist_creates_file_with_label_and_run_at_load() {
        let p = tmp_plist("write");
        let exe = Path::new("/usr/local/bin/text-chisel");
        write_plist(&p, exe).unwrap();
        let content = std::fs::read_to_string(&p).unwrap();
        assert!(content.contains("com.textchisel.app"));
        assert!(content.contains("RunAtLoad"));
        assert!(content.contains("/usr/local/bin/text-chisel"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn write_plist_xml_escapes_exe_path() {
        let p = tmp_plist("escape");
        let exe = Path::new("/Applications/Apps & Stuff/text-chisel");
        write_plist(&p, exe).unwrap();
        let content = std::fs::read_to_string(&p).unwrap();
        assert!(content.contains("Apps &amp; Stuff"));
        assert!(!content.contains("Apps & Stuff"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn write_plist_creates_parent_directory() {
        let dir = std::env::temp_dir().join("text-chisel-test-mkdir");
        let p = dir.join("test.plist");
        let exe = Path::new("/usr/local/bin/text-chisel");
        write_plist(&p, exe).unwrap();
        assert!(p.exists());
        let _ = std::fs::remove_file(&p);
        let _ = std::fs::remove_dir(&dir);
    }

    // --- remove_plist ---

    #[test]
    fn remove_plist_deletes_existing_file() {
        let p = tmp_plist("remove");
        std::fs::write(&p, "test").unwrap();
        remove_plist(&p).unwrap();
        assert!(!p.exists());
    }

    #[test]
    fn remove_plist_noop_when_file_absent() {
        let p = tmp_plist("no-file");
        let _ = std::fs::remove_file(&p);
        assert!(remove_plist(&p).is_ok());
    }

    // --- roundtrip (manual) ---

    #[test]
    #[ignore = "Writes and removes ~/Library/LaunchAgents/com.textchisel.app.plist on the real filesystem. Run manually to verify enable/disable works correctly."]
    fn test_launch_at_login_roundtrip() {
        if is_enabled() {
            disable().expect("failed to disable before test");
        }
        assert!(!is_enabled(), "should be disabled before enable");

        enable().expect("enable failed");
        assert!(is_enabled(), "should be enabled after enable");

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
