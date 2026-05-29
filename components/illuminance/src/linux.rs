async fn read_illuminance(device_path: Option<&String>) -> Result<f64, String> {
    use std::{fs, path::Path};

    // If a specific device path is provided, use it
    if let Some(path) = device_path {
        return read_linux_illuminance_from_path(path).await;
    }

    // Auto-detect light sensor devices
    let iio_base = "/sys/bus/iio/devices";
    if Path::new(iio_base).exists() {
        let entries = fs::read_dir(iio_base)
            .map_err(|e| format!("Failed to read IIO devices directory: {}", e))?;

        for entry in entries.flatten() {
            let device_path = entry.path();
            if let Some(device_name) = device_path.file_name() {
                if device_name.to_string_lossy().starts_with("iio:device") {
                    // Check if this device has illuminance capabilities
                    let illuminance_raw_path = device_path.join("in_illuminance_raw");
                    let illuminance_input_path = device_path.join("in_illuminance_input");

                    if illuminance_raw_path.exists() {
                        if let Ok(value) = read_linux_illuminance_from_path(
                            illuminance_raw_path.to_string_lossy().as_ref(),
                        )
                        .await
                        {
                            return Ok(value);
                        }
                    } else if illuminance_input_path.exists() {
                        if let Ok(value) = read_linux_illuminance_from_path(
                            illuminance_input_path.to_string_lossy().as_ref(),
                        )
                        .await
                        {
                            return Ok(value);
                        }
                    }
                }
            }
        }
    }

    // Try alternative paths for some hardware
    let hwmon_paths = [
        "/sys/class/hwmon/hwmon0/device/als",
        "/sys/class/hwmon/hwmon1/device/als",
        "/sys/devices/platform/applesmc.768/light",
    ];

    for path in &hwmon_paths {
        if Path::new(path).exists() {
            if let Ok(value) = read_linux_illuminance_from_path(path).await {
                return Ok(value);
            }
        }
    }

    Err("No light sensor found on this system".to_string())
}

async fn read_linux_illuminance_from_path(path: &str) -> Result<f64, String> {
    use std::fs;

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read light sensor from {}: {}", path, e))?;

    let value: f64 = content.trim().parse().map_err(|e| {
        format!(
            "Failed to parse light sensor value '{}': {}",
            content.trim(),
            e
        )
    })?;

    Ok(value)
}
