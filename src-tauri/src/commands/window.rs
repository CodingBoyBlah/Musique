use crate::errors::AppError;
use crate::state::AppState;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use tauri::Manager;
use tauri::{AppHandle, State};

/// whether a native backdrop material (mica/acrylic/vibrancy) is on. the
/// frontend stays transparent when true, otherwise it paints an opaque dark fallback
/// so the ui never ends up white on white
#[tauri::command]
pub fn get_backdrop_active(state: State<'_, AppState>) -> bool {
    state.backdrop_active
}

/// swap the window backdrop material at runtime between mica and acrylic.
/// clears the other material first so they dont stack on top of each other
#[tauri::command]
pub async fn set_window_effect(app: AppHandle, mode: String) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{apply_acrylic, apply_mica, clear_acrylic, clear_mica};

        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::NotFound("main window".into()))?;

        match mode.as_str() {
            "acrylic" => {
                let _ = clear_mica(&window);
                apply_acrylic(&window, Some((10, 10, 12, 110)))
                    .map_err(|e| AppError::InvalidInput(format!("acrylic: {e}")))?;
            }
            "mica" => {
                let _ = clear_acrylic(&window);
                apply_mica(&window, Some(true))
                    .map_err(|e| AppError::InvalidInput(format!("mica: {e}")))?;
            }
            "none" => {
                // drop both materials, the frontend paints its own solid base
                // (see backdropScrim) so the transparent window stays opaque
                let _ = clear_mica(&window);
                let _ = clear_acrylic(&window);
            }
            other => {
                return Err(AppError::InvalidInput(format!("unknown effect: {other}")));
            }
        }
    }

    // macOS: the two options are Vibrancy (any non-"none" value) and No material.
    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{
            apply_vibrancy, clear_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
        };

        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::NotFound("main window".into()))?;

        match mode.as_str() {
            "none" => {
                // frontend paints its own solid base (see backdropScrim)
                let _ = clear_vibrancy(&window);
            }
            _ => {
                // vibrancy on - try materials in order, keep the first that applies
                for material in [
                    NSVisualEffectMaterial::HudWindow,
                    NSVisualEffectMaterial::UnderWindowBackground,
                    NSVisualEffectMaterial::WindowBackground,
                ] {
                    if apply_vibrancy(&window, material, Some(NSVisualEffectState::Active), None)
                        .is_ok()
                    {
                        break;
                    }
                }
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (&app, &mode);
    }

    Ok(())
}
