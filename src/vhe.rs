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

// Whoever sits at this computer wins the mouse: remote moves and presses are
// dropped for a moment after any physical (non-injected) local mouse input.
#[cfg(windows)]
const LOCAL_MOUSE_PRIORITY_MS: u64 = 1_000;

#[cfg(windows)]
static LAST_LOCAL_MOUSE_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(windows)]
fn now_ms() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    // Offset by 1 so a recorded time is never the "never moved" value 0.
    START.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64 + 1
}

#[cfg(windows)]
pub fn local_mouse_busy() -> bool {
    static HOOK: std::sync::Once = std::sync::Once::new();
    HOOK.call_once(|| {
        now_ms();
        std::thread::spawn(local_mouse_hook_thread);
    });
    let last = LAST_LOCAL_MOUSE_MS.load(std::sync::atomic::Ordering::Relaxed);
    last != 0 && now_ms().saturating_sub(last) < LOCAL_MOUSE_PRIORITY_MS
}

#[cfg(windows)]
unsafe extern "system" fn local_mouse_hook_proc(
    code: i32,
    wparam: winapi::shared::minwindef::WPARAM,
    lparam: winapi::shared::minwindef::LPARAM,
) -> winapi::shared::minwindef::LRESULT {
    use winapi::um::winuser::{CallNextHookEx, LLMHF_INJECTED, MSLLHOOKSTRUCT};
    if code >= 0 && lparam != 0 {
        let info = &*(lparam as *const MSLLHOOKSTRUCT);
        if info.flags & LLMHF_INJECTED == 0 {
            LAST_LOCAL_MOUSE_MS.store(now_ms(), std::sync::atomic::Ordering::Relaxed);
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

#[cfg(windows)]
fn local_mouse_hook_thread() {
    use winapi::um::winuser::{
        GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, MSG, WH_MOUSE_LL,
    };
    unsafe {
        let hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(local_mouse_hook_proc),
            std::ptr::null_mut(),
            0,
        );
        if hook.is_null() {
            log::error!("VHE: failed to install local mouse hook");
            return;
        }
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {}
        UnhookWindowsHookEx(hook);
    }
}
