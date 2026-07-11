use base64::Engine;
use std::path::{Path, PathBuf};
use voice_type::config;

#[tauri::command]
pub fn import_background_image(path: String) -> Result<String, String> {
    import_background_asset(&path, "default-background")
}

#[tauri::command]
pub fn import_hud_background_image(path: String) -> Result<String, String> {
    import_background_asset(&path, "hud-background")
}

fn import_background_asset(path: &str, file_stem: &str) -> Result<String, String> {
    let source = PathBuf::from(path.trim());
    if !source.exists() {
        return Err("Selected background image does not exist".to_string());
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "Background image must have a valid file extension".to_string())?;

    let allowed_extensions = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];
    if !allowed_extensions.contains(&extension.as_str()) {
        return Err("Unsupported background image format".to_string());
    }

    // Read a bounded prefix and verify the file is actually a decodable image.
    // This prevents importing arbitrary files (e.g. secrets renamed to .png)
    // which could later be exfiltrated via load_background_image_data_url.
    const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024; // 32 MB
    let metadata = std::fs::metadata(&source)
        .map_err(|error| format!("Failed to read file metadata: {}", error))?;
    if metadata.len() > MAX_IMAGE_BYTES {
        return Err("Background image exceeds the 32 MB size limit".to_string());
    }

    let bytes = std::fs::read(&source)
        .map_err(|error| format!("Failed to read background image: {}", error))?;
    if !is_valid_image_header(&bytes, &extension) {
        return Err(
            "File does not contain valid image data matching its extension".to_string(),
        );
    }

    let destination_dir = config::backgrounds_dir()
        .map_err(|error| format!("Failed to prepare backgrounds directory: {}", error))?;

    remove_old_background_assets(&destination_dir, file_stem)
        .map_err(|error| format!("Failed to replace existing background image: {}", error))?;

    let destination = destination_dir.join(format!("{}.{}", file_stem, extension));
    std::fs::write(&destination, &bytes)
        .map_err(|error| format!("Failed to import background image: {}", error))?;

    destination
        .into_os_string()
        .into_string()
        .map_err(|_| "Imported background path is not valid UTF-8".to_string())
}

#[tauri::command]
pub fn load_background_image_data_url(path: String) -> Result<String, String> {
    let requested_path = PathBuf::from(path.trim());
    if !requested_path.exists() {
        return Err("Imported background image does not exist".to_string());
    }

    let backgrounds_dir = config::backgrounds_dir()
        .map_err(|error| format!("Failed to resolve backgrounds directory: {}", error))?;

    let canonical_requested = requested_path
        .canonicalize()
        .map_err(|error| format!("Failed to resolve imported background path: {}", error))?;
    let canonical_backgrounds = backgrounds_dir
        .canonicalize()
        .map_err(|error| format!("Failed to resolve backgrounds directory: {}", error))?;

    if !canonical_requested.starts_with(&canonical_backgrounds) {
        return Err(
            "Background image path is outside the app-controlled backgrounds folder".to_string(),
        );
    }

    let extension = canonical_requested
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "Imported background image must have a valid file extension".to_string())?;

    let mime = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        _ => return Err("Unsupported imported background image format".to_string()),
    };

    let bytes = std::fs::read(&canonical_requested)
        .map_err(|error| format!("Failed to read imported background image: {}", error))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}

fn remove_old_background_assets(directory: &Path, file_stem: &str) -> Result<(), std::io::Error> {
    if !directory.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let matches_stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(|value| value == file_stem)
            .unwrap_or(false);

        if path.is_file() && matches_stem {
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}

/// Verify the byte stream starts with a recognized image magic-number sequence
/// that matches the declared extension.  This blocks importing arbitrary files
/// that merely carry an image extension.
fn is_valid_image_header(bytes: &[u8], extension: &str) -> bool {
    if bytes.len() < 12 {
        return false;
    }
    match extension {
        "png" => bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        "jpg" | "jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "bmp" => bytes.starts_with(&[0x42, 0x4D]), // "BM"
        "webp" => {
            // RIFF....WEBP
            bytes.len() >= 12
                && &bytes[0..4] == b"RIFF"
                && &bytes[8..12] == b"WEBP"
        }
        _ => false,
    }
}
