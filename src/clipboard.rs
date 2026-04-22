use anyhow::{Context, Result};
use arboard::Clipboard;
use tracing::{debug, error, warn};

/// Copy text to system clipboard
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            clipboard
                .set_text(text.to_string())
                .context("Failed to set clipboard text")?;
            
            debug!("Copied to clipboard: {} chars", text.len());
            Ok(())
        }
        Err(e) => {
            // On Linux, try to use xclip/xsel as fallback
            #[cfg(target_os = "linux")]
            {
                warn!("arboard failed, trying xclip fallback: {}", e);
                fallback_linux_clipboard(text)
            }
            
            #[cfg(not(target_os = "linux"))]
            {
                Err(e).context("Failed to initialize clipboard")
            }
        }
    }
}

/// Copy text with success notification format
pub fn copy_with_prefix(prefix: &str, text: &str) -> Result<()> {
    let content = format!("{}: {}", prefix, text);
    copy_to_clipboard(&content)
}

/// Copy policy ID from an asset
pub fn copy_policy_id(policy_id: &str) -> Result<()> {
    copy_to_clipboard(policy_id)
}

/// Copy raw CBOR hex
pub fn copy_raw_cbor(cbor: &str) -> Result<()> {
    copy_to_clipboard(cbor)
}

/// Copy transaction hash
pub fn copy_tx_hash(hash: &str) -> Result<()> {
    copy_to_clipboard(hash)
}

/// Copy address
pub fn copy_address(address: &str) -> Result<()> {
    copy_to_clipboard(address)
}

/// Copy decoded Plutus data as formatted text
pub fn copy_plutus_data(data: &crate::decoder::PlutusNode) -> Result<()> {
    let formatted = data.to_string_pretty();
    copy_to_clipboard(&formatted)
}

/// Copy asset information
pub fn copy_asset_info(policy_id: &str, asset_name: &str, amount: u64) -> Result<()> {
    let info = if policy_id == "ada" {
        let ada = amount as f64 / 1_000_000.0;
        format!("₳ {:.6}", ada)
    } else {
        format!("Policy ID: {}\nAsset Name: {}\nAmount: {}", 
            policy_id, 
            if asset_name.is_empty() { "(none)" } else { asset_name },
            amount
        )
    };
    copy_to_clipboard(&info)
}

/// Try to get clipboard text (for paste functionality)
pub fn get_clipboard_text() -> Result<String> {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            clipboard
                .get_text()
                .context("Failed to get clipboard text")
        }
        Err(e) => {
            #[cfg(target_os = "linux")]
            {
                warn!("arboard failed, trying xclip fallback for paste: {}", e);
                fallback_linux_paste()
            }
            
            #[cfg(not(target_os = "linux"))]
            {
                Err(e).context("Failed to initialize clipboard for paste")
            }
        }
    }
}

/// Check if clipboard is available
pub fn is_clipboard_available() -> bool {
    Clipboard::new().is_ok()
}

/// Clear clipboard (security feature for sensitive data)
pub fn clear_clipboard() -> Result<()> {
    if let Ok(mut clipboard) = Clipboard::new() {
        clipboard.set_text("").context("Failed to clear clipboard")?;
        debug!("Clipboard cleared");
    }
    Ok(())
}

/// Copy with automatic truncation for very long content
pub fn copy_truncated(text: &str, max_len: usize) -> Result<()> {
    let content = if text.len() > max_len {
        format!("{}... (truncated)", &text[..max_len])
    } else {
        text.to_string()
    };
    copy_to_clipboard(&content)
}

#[cfg(target_os = "linux")]
fn fallback_linux_clipboard(text: &str) -> Result<()> {
    use std::process::Command;
    use std::io::Write;
    
    // Try xclip first
    let xclip_result = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()
        });
    
    if xclip_result.is_ok() {
        debug!("Copied using xclip fallback");
        return Ok(());
    }
    
    // Try xsel as second option
    let xsel_result = Command::new("xsel")
        .args(["--clipboard", "--input"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()
        });
    
    if xsel_result.is_ok() {
        debug!("Copied using xsel fallback");
        return Ok(());
    }
    
    Err(anyhow::anyhow!("No clipboard utility available. Install xclip or xsel."))
}

#[cfg(target_os = "linux")]
fn fallback_linux_paste() -> Result<String> {
    use std::process::Command;
    
    // Try xclip first
    if let Ok(output) = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
    {
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .context("Invalid UTF-8 in clipboard");
        }
    }
    
    // Try xsel as second option
    if let Ok(output) = Command::new("xsel")
        .args(["--clipboard", "--output"])
        .output()
    {
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .context("Invalid UTF-8 in clipboard");
        }
    }
    
    Err(anyhow::anyhow!("No clipboard utility available. Install xclip or xsel."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_availability() {
        // Just check that it doesn't panic
        let _ = is_clipboard_available();
    }

    #[test]
    fn test_copy_formats() {
        // Skip actual clipboard operations in CI
        if std::env::var("CI").is_ok() {
            return;
        }
        
        // These might fail on headless systems, so we ignore errors
        let _ = copy_with_prefix("test", "value");
        let _ = copy_policy_id("test_policy");
        let _ = copy_raw_cbor("abcdef");
        let _ = copy_tx_hash("1234567890abcdef");
        let _ = copy_address("addr1test");
    }

    #[test]
    fn test_copy_truncated() {
        let long_text = "a".repeat(1000);
        let result = copy_truncated(&long_text, 100);
        if std::env::var("CI").is_ok() {
            return;
        }
        // Should succeed or fail gracefully
        let _ = result;
    }
}