//! `ocean-login` — run a provider OAuth sign-in without the TUI.
//!
//! The TUI is the only place a login could be started, which makes it impossible
//! to sign in from a non-interactive context (scripts, remote shells, an agent
//! driving setup) — and a full-screen TUI is exactly what such a caller cannot
//! host. Everything needed is already public on `ocean_oauth`: `begin()` returns
//! the authorize URL and stands up the loopback callback server, `finish()`
//! waits for the redirect and writes the credential block. This just wires those
//! two to stdout.
//!
//!   ocean-login claude     # Claude / Fable  -> `claude-code` block
//!   ocean-login codex      # OpenAI Codex    -> `openai-codex` block
//!
//! Prints the URL, then blocks until the browser redirect lands.

use anyhow::{Result, anyhow};
use ocean_oauth::{OAuthProvider, begin};

#[tokio::main]
async fn main() -> Result<()> {
    let arg = std::env::args().nth(1).unwrap_or_default();
    let provider = match arg.to_ascii_lowercase().as_str() {
        "claude" | "fable" | "claude-code" => OAuthProvider::Claude,
        "codex" | "openai-codex" => OAuthProvider::Codex,
        "" => return Err(anyhow!("usage: ocean-login <claude|codex>")),
        other => return Err(anyhow!("unknown provider {other:?} — use `claude` or `codex`")),
    };

    let session = begin(provider, None).await?;

    // stdout is the machine-readable surface: a caller can grab the URL with
    // `head -1` and hand it straight to a browser. Prose goes to stderr.
    println!("{}", session.authorize_url);
    eprintln!("\n  provider : {}", provider.label());
    eprintln!("  shortcut : {}", session.launch_url);
    eprintln!("\n  Open the URL above and complete the sign-in.");
    eprintln!("  Waiting for the browser redirect…\n");

    let outcome = session.finish().await?;

    eprintln!("✓ {} login complete", outcome.provider.label());
    eprintln!("  auth file : {}", outcome.auth_file.display());
    eprintln!("  block     : {}", outcome.provider.auth_json_key());
    if let Some(id) = outcome.account_id.as_deref() {
        eprintln!("  account   : {id}");
    }
    eprintln!("  expires   : {} (unix ms)", outcome.expires_ms);
    eprintln!("\n  Verify with: curl -s http://127.0.0.1:4780/v1/models");
    Ok(())
}
