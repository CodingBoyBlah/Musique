use crate::errors::AppError;
use crate::state::AppState;
use tauri::{AppHandle, State};
#[cfg(target_os = "windows")]
use tauri::Manager;

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

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (&app, &mode);
    }

    Ok(())
}
