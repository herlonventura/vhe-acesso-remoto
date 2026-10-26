// VHE Acesso Remoto: peer-to-peer over Tailscale only, no ID/relay servers.
use hbb_common::config;

pub const APP_NAME: &str = "VHE-Acesso-Remoto";

// Tailscale CGNAT range and its IPv6 ULA prefix.
const TAILSCALE_NETS: &str = "100.64.0.0/10,fd7a:115c:a1e0::/48";

pub fn apply_settings() {
    *config::APP_NAME.write().unwrap() = APP_NAME.to_owned();
    let mut settings = config::OVERWRITE_SETTINGS.write().unwrap();
    for (k, v) in [
        ("custom-rendezvous-server", "127.0.0.1"),
        ("relay-server", "127.0.0.1"),
        ("api-server", "http://127.0.0.1"),
        ("direct-server", "Y"),
        ("direct-access-port", "21118"),
        ("whitelist", TAILSCALE_NETS),
        ("verification-method", "use-permanent-password"),
        ("approve-mode", "password"),
        ("enable-lan-discovery", "N"),
        ("enable-check-update", "N"),
        ("allow-auto-update", "N"),
        ("hide-server-settings", "Y"),
        ("hide-proxy-settings", "Y"),
        ("hide-websocket-settings", "Y"),
    ] {
        settings.insert(k.to_owned(), v.to_owned());
    }
}

// Installs the official, Tailscale-signed MSI at install time; the MSI itself is never bundled.
#[cfg(windows)]
pub fn tailscale_install_cmd() -> &'static str {
    // The install script sets PATH to System32 only, so PowerShell needs its full path.
    "\"%SystemRoot%\\System32\\WindowsPowerShell\\v1.0\\powershell.exe\" -NoProfile -ExecutionPolicy Bypass -Command \"if (-not (Test-Path 'C:\\Program Files\\Tailscale\\tailscale-ipn.exe')) { [Net.ServicePointManager]::SecurityProtocol = 'Tls12'; $f = Join-Path $env:ProgramData 'vhe-tailscale-setup.msi'; try { Invoke-WebRequest -UseBasicParsing -Uri 'https://pkgs.tailscale.com/stable/tailscale-setup-latest-amd64.msi' -OutFile $f; $s = Get-AuthenticodeSignature $f; if ($s.Status -eq 'Valid' -and $s.SignerCertificate.Subject -like 'CN=Tailscale Inc.,*') { Start-Process msiexec.exe -Wait -ArgumentList ('/i ' + $f + ' /quiet /norestart TS_UNATTENDEDMODE=always') } } catch {} finally { Remove-Item $f -ErrorAction SilentlyContinue } }\""
}
