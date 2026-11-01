use crate::audio;
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

/// Managed tray icon handle for dynamic menu/tooltip updates.
pub struct TrayHandle {
    pub tray: Mutex<tauri::tray::TrayIcon>,
}

struct TrayLabels {
    open_home: &'static str,
    history: &'static str,
    add_word: &'static str,
    settings: &'static str,
    microphone: &'static str,
    system_default: &'static str,
    version: &'static str,
    quit: &'static str,
}

fn get_tray_labels(lang: &str) -> TrayLabels {
    match lang {
        "zh" => TrayLabels {
            open_home: "打开 Talkey 主页",
            history: "查看历史记录",
            add_word: "向词典添加单词",
            settings: "设置...",
            microphone: "选择麦克风",
            system_default: "系统默认",
            version: "版本",
            quit: "退出 Talkey",
        },
        "ja" => TrayLabels {
            open_home: "Talkey ホームを開く",
            history: "履歴を表示",
            add_word: "辞書に単語を追加",
            settings: "設定...",
            microphone: "マイクを選択",
            system_default: "システムのデフォルト",
            version: "バージョン",
            quit: "Talkey を終了",
        },
        "ko" => TrayLabels {
            open_home: "Talkey 홈 열기",
            history: "기록 보기",
            add_word: "사전에 단어 추가",
            settings: "설정...",
            microphone: "마이크 선택",
            system_default: "시스템 기본값",
            version: "버전",
            quit: "Talkey 종료",
        },
        "fr" => TrayLabels {
            open_home: "Ouvrir l'accueil Talkey",
            history: "Voir l'historique",
            add_word: "Ajouter un mot au dictionnaire",
            settings: "Paramètres...",
            microphone: "Choisir le micro",
            system_default: "Par défaut du système",
            version: "Version",
            quit: "Quitter Talkey",
        },
        "de" => TrayLabels {
            open_home: "Talkey-Startseite öffnen",
            history: "Verlauf anzeigen",
            add_word: "Wort zum Wörterbuch hinzufügen",
            settings: "Einstellungen...",
            microphone: "Mikrofon wählen",
            system_default: "Systemstandard",
            version: "Version",
            quit: "Talkey beenden",
        },
        "es" => TrayLabels {
            open_home: "Abrir inicio de Talkey",
            history: "Ver historial",
            add_word: "Añadir palabra al diccionario",
            settings: "Configuración...",
            microphone: "Elegir micrófono",
            system_default: "Predeterminado del sistema",
            version: "Versión",
            quit: "Salir de Talkey",
        },
        "pt" => TrayLabels {
            open_home: "Abrir início do Talkey",
            history: "Ver histórico",
            add_word: "Adicionar palavra ao dicionário",
            settings: "Configurações...",
            microphone: "Escolher microfone",
            system_default: "Padrão do sistema",
            version: "Versão",
            quit: "Sair do Talkey",
        },
        "ru" => TrayLabels {
            open_home: "Открыть главную Talkey",
            history: "Показать историю",
            add_word: "Добавить слово в словарь",
            settings: "Настройки...",
            microphone: "Выбрать микрофон",
            system_default: "Системный по умолчанию",
            version: "Версия",
            quit: "Выйти из Talkey",
        },
        "it" => TrayLabels {
            open_home: "Apri la home di Talkey",
            history: "Visualizza cronologia",
            add_word: "Aggiungi parola al dizionario",
            settings: "Impostazioni...",
            microphone: "Scegli microfono",
            system_default: "Predefinito di sistema",
            version: "Versione",
            quit: "Esci da Talkey",
        },
        _ => TrayLabels {
            open_home: "Open Talkey Home",
            history: "View History",
            add_word: "Add Word to Dictionary",
            settings: "Settings...",
            microphone: "Select Microphone",
            system_default: "System Default",
            version: "Version",
            quit: "Quit Talkey",
        },
    }
}

fn app_config_value(app: &tauri::AppHandle) -> Option<serde_json::Value> {
    app.store("settings.json")
        .ok()
        .and_then(|store| store.get("app_config"))
}

fn tray_language(app: &tauri::AppHandle) -> String {
    app_config_value(app)
        .and_then(|config| {
            config
                .get("ui_language")
                .and_then(|value| value.as_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "en".to_string())
}

fn tray_input_device(app: &tauri::AppHandle) -> String {
    app_config_value(app)
        .and_then(|config| {
            config
                .get("input_device")
                .and_then(|value| value.as_str())
                .map(String::from)
        })
        .unwrap_or_default()
}

/// Input devices shown in the microphone submenu.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MicrophoneSnapshot {
    devices: Vec<String>,
    system_default: Option<String>,
}

impl MicrophoneSnapshot {
    fn read() -> Self {
        Self {
            devices: audio::capture::input_device_names(),
            system_default: audio::capture::system_default_input_device_name(),
        }
    }
}

/// Last device list, so menu rebuilds on pipeline state changes do not
/// enumerate audio devices on the recording path.
static MICROPHONES: Mutex<Option<MicrophoneSnapshot>> = Mutex::new(None);

fn cached_microphones() -> MicrophoneSnapshot {
    let mut cached = MICROPHONES.lock().unwrap_or_else(|e| e.into_inner());
    cached.get_or_insert_with(MicrophoneSnapshot::read).clone()
}

/// Watches for microphones being plugged in or removed and rebuilds the tray
/// menu when the list changes.
pub fn spawn_microphone_watcher(app: tauri::AppHandle) {
    std::thread::Builder::new()
        .name("tray-microphones".to_string())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let next = MicrophoneSnapshot::read();
            let changed = {
                let mut cached = MICROPHONES.lock().unwrap_or_else(|e| e.into_inner());
                let changed = cached.as_ref() != Some(&next);
                *cached = Some(next);
                changed
            };
            if changed {
                refresh_tray(&app);
            }
        })
        .map(|_| ())
        .unwrap_or_else(|e| tracing::warn!("Failed to start the microphone watcher: {}", e));
}

/// Menu id prefix for a microphone; the rest of the id is the device name.
pub const MICROPHONE_ID_PREFIX: &str = "microphone:";
/// Menu id for recording from the system default microphone.
pub const MICROPHONE_DEFAULT_ID: &str = "microphone-default";

fn microphone_items(
    app: &tauri::AppHandle,
    labels: &TrayLabels,
    microphones: &MicrophoneSnapshot,
    selected: &str,
) -> Result<Vec<CheckMenuItem<tauri::Wry>>, Box<dyn std::error::Error>> {
    let selected_connected = microphones.devices.iter().any(|name| name == selected);
    let default_label = match &microphones.system_default {
        Some(name) => format!("{} ({})", labels.system_default, name),
        None => labels.system_default.to_string(),
    };
    let mut items = vec![CheckMenuItem::with_id(
        app,
        MICROPHONE_DEFAULT_ID,
        default_label,
        true,
        !selected_connected,
        None::<&str>,
    )?];
    for name in &microphones.devices {
        items.push(CheckMenuItem::with_id(
            app,
            format!("{MICROPHONE_ID_PREFIX}{name}"),
            name,
            true,
            selected_connected && name == selected,
            None::<&str>,
        )?);
    }
    Ok(items)
}

/// Build (or rebuild) the system tray menu.
pub fn build_tray_menu(
    app: &tauri::AppHandle,
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let labels = get_tray_labels(&tray_language(app));

    let open_home = MenuItem::with_id(app, "open_home", labels.open_home, true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", labels.history, true, None::<&str>)?;
    let add_word = MenuItem::with_id(app, "add_word", labels.add_word, true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", labels.settings, true, Some("CmdOrCtrl+,"))?;

    let microphone_entries =
        microphone_items(app, &labels, &cached_microphones(), &tray_input_device(app))?;
    let mic_sep = PredefinedMenuItem::separator(app)?;
    let mut microphone_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = Vec::new();
    for (index, item) in microphone_entries.iter().enumerate() {
        microphone_refs.push(item);
        if index == 0 && microphone_entries.len() > 1 {
            microphone_refs.push(&mic_sep);
        }
    }
    let microphone =
        Submenu::with_id_and_items(app, "microphone", labels.microphone, true, &microphone_refs)?;

    let sep2 = PredefinedMenuItem::separator(app)?;
    let version = MenuItem::with_id(
        app,
        "version",
        format!("{} {}", labels.version, app.package_info().version),
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, Some("CmdOrCtrl+Q"))?;

    let menu = Menu::with_items(
        app,
        &[
            &open_home,
            &history,
            &add_word,
            &sep1,
            &settings,
            &microphone,
            &sep2,
            &version,
            &quit,
        ],
    )?;
    Ok(menu)
}

/// Rebuild the tray menu, e.g. after the language or microphone changes.
pub fn refresh_tray(app: &tauri::AppHandle) {
    if let Some(tray_handle) = app.try_state::<TrayHandle>() {
        if let Ok(tray) = tray_handle.tray.lock() {
            if let Ok(menu) = build_tray_menu(app) {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_labels_match_the_typeless_style_menu() {
        let labels = get_tray_labels("ko");

        assert_eq!(labels.open_home, "Talkey 홈 열기");
        assert_eq!(labels.add_word, "사전에 단어 추가");
        assert_eq!(labels.settings, "설정...");
        assert_eq!(labels.quit, "Talkey 종료");
    }

    #[test]
    fn unknown_language_falls_back_to_english_labels() {
        let labels = get_tray_labels("xx");

        assert_eq!(labels.microphone, "Select Microphone");
        assert_eq!(labels.quit, "Quit Talkey");
    }
}
