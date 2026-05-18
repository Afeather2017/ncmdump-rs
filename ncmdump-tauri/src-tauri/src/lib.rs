use bilibili_api::auth::BiliSession;
use netease_api::auth::Session as NeteaseSession;
use serde::{Deserialize, Serialize};
use tauri::Manager;

const NETEASE_LOGIN_URL: &str = "https://music.163.com/";
const BILIBILI_LOGIN_URL: &str = "https://www.bilibili.com/";
const MAIN_WINDOW_LABEL: &str = "main";

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Netease,
    Bilibili,
}

#[derive(Debug, Serialize)]
struct ProviderStatus {
    provider: &'static str,
    is_logged_in: bool,
    session_path: String,
    summary: String,
}

fn main_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "main window not found".to_string())
}

fn config_dir() -> Result<std::path::PathBuf, String> {
    dirs::config_dir().ok_or_else(|| "cannot determine config directory".to_string())
}

fn netease_session_path() -> Result<String, String> {
    Ok(config_dir()?
        .join("ncmdump")
        .join("session.json")
        .display()
        .to_string())
}

fn bilibili_session_path() -> Result<String, String> {
    Ok(config_dir()?
        .join("ncmdump")
        .join("bilibili_session.json")
        .display()
        .to_string())
}

fn load_netease_status() -> Result<ProviderStatus, String> {
    let session = NeteaseSession::load().map_err(|e| e.to_string())?;
    Ok(ProviderStatus {
        provider: "netease",
        is_logged_in: session.is_logged_in(),
        session_path: netease_session_path()?,
        summary: if session.is_logged_in() {
            "MUSIC_U is present".to_string()
        } else {
            "No MUSIC_U saved".to_string()
        },
    })
}

fn load_bilibili_status() -> Result<ProviderStatus, String> {
    let session = BiliSession::load().map_err(|e| e.to_string())?;
    let mut fields = Vec::new();
    if session.sessdata.as_ref().is_some_and(|v| !v.is_empty()) {
        fields.push("SESSDATA");
    }
    if session.bili_jct.as_ref().is_some_and(|v| !v.is_empty()) {
        fields.push("bili_jct");
    }
    if session.dede_user_id.as_ref().is_some_and(|v| !v.is_empty()) {
        fields.push("DedeUserID");
    }
    if session.buvid3.as_ref().is_some_and(|v| !v.is_empty()) {
        fields.push("buvid3");
    }
    if session.buvid4.as_ref().is_some_and(|v| !v.is_empty()) {
        fields.push("buvid4");
    }

    let is_logged_in = session.is_logged_in();
    Ok(ProviderStatus {
        provider: "bilibili",
        is_logged_in,
        session_path: bilibili_session_path()?,
        summary: if fields.is_empty() {
            "No Bilibili session cookies saved".to_string()
        } else {
            format!("Saved cookies: {}", fields.join(", "))
        },
    })
}

fn load_status(provider: Provider) -> Result<ProviderStatus, String> {
    match provider {
        Provider::Netease => load_netease_status(),
        Provider::Bilibili => load_bilibili_status(),
    }
}

fn extract_music_u_from_webview(app: &tauri::AppHandle) -> Result<String, String> {
    let browser = main_window(app)?;
    let cookies = browser
        .cookies()
        .map_err(|e| format!("failed to read browser cookies: {e}"))?;

    for cookie in cookies {
        let domain = cookie
            .domain_raw()
            .or_else(|| cookie.domain())
            .unwrap_or("")
            .to_ascii_lowercase();

        if !domain.contains("music.163.com") && !domain.contains(".163.com") {
            continue;
        }

        if cookie.name() == "MUSIC_U" {
            let value = cookie.value().to_string();
            if !value.is_empty() {
                return Ok(value);
            }
        }
    }

    Err("MUSIC_U cookie not found in the Tauri webview. Log in on music.163.com in this window first.".to_string())
}

fn extract_bilibili_session_from_webview(app: &tauri::AppHandle) -> Result<BiliSession, String> {
    let browser = main_window(app)?;
    let cookies = browser
        .cookies()
        .map_err(|e| format!("failed to read browser cookies: {e}"))?;

    let mut session = BiliSession::default();

    for cookie in cookies {
        let domain = cookie
            .domain_raw()
            .or_else(|| cookie.domain())
            .unwrap_or("")
            .to_ascii_lowercase();

        if !domain.contains("bilibili.com") {
            continue;
        }

        let value = cookie.value().to_string();
        if value.is_empty() {
            continue;
        }

        match cookie.name() {
            "SESSDATA" => session.sessdata = Some(value),
            "bili_jct" => session.bili_jct = Some(value),
            "DedeUserID" => session.dede_user_id = Some(value),
            "buvid3" => session.buvid3 = Some(value),
            "buvid4" => session.buvid4 = Some(value),
            _ => {}
        }
    }

    if !session.is_logged_in() {
        return Err(
            "SESSDATA cookie not found in the Tauri webview. Log in on bilibili.com in this window first."
                .to_string(),
        );
    }

    Ok(session)
}

#[tauri::command]
fn open_login(app: tauri::AppHandle, provider: Provider) -> Result<(), String> {
    let window = main_window(&app)?;
    let url = match provider {
        Provider::Netease => NETEASE_LOGIN_URL,
        Provider::Bilibili => BILIBILI_LOGIN_URL,
    };

    window.set_focus().ok();
    window
        .navigate(url::Url::parse(url).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn capture_login_cookie(
    app: tauri::AppHandle,
    provider: Provider,
) -> Result<ProviderStatus, String> {
    match provider {
        Provider::Netease => {
            let music_u = extract_music_u_from_webview(&app)?;
            let session = NeteaseSession {
                music_u: Some(music_u),
            };
            session.save().map_err(|e| e.to_string())?;
            load_netease_status()
        }
        Provider::Bilibili => {
            let session = extract_bilibili_session_from_webview(&app)?;
            session.save().map_err(|e| e.to_string())?;
            load_bilibili_status()
        }
    }
}

#[tauri::command]
fn login_status(provider: Provider) -> Result<ProviderStatus, String> {
    load_status(provider)
}

#[tauri::command]
fn logout(provider: Provider) -> Result<ProviderStatus, String> {
    match provider {
        Provider::Netease => {
            NeteaseSession::clear().map_err(|e| e.to_string())?;
            load_netease_status()
        }
        Provider::Bilibili => {
            BiliSession::clear().map_err(|e| e.to_string())?;
            load_bilibili_status()
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_login,
            capture_login_cookie,
            login_status,
            logout
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
