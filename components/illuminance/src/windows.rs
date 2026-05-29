async fn read_illuminance(_device_path: Option<&String>) -> Result<f64, String> {
    use windows::{
        Win32::Devices::Sensors::{
            ISensorDataReport, ISensorManager, SENSOR_DATA_TYPE_LIGHT_LEVEL_LUX,
            SENSOR_TYPE_AMBIENT_LIGHT,
        },
        Win32::System::Com::StructuredStorage::PropVariantToDouble,
        Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        },
    };

    unsafe {
        // Initialize COM

        use windows::Win32::{Devices::Sensors::SensorManager, System::Com::COINIT_MULTITHREADED};
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            return Err("Failed to initialize COM".to_string());
        }

        // Create a SensorManager instance
        let sensor_manager: ISensorManager =
            CoCreateInstance(&SensorManager, None, CLSCTX_INPROC_SERVER).map_err(|e| {
                CoUninitialize();
                format!("Failed to create SensorManager instance: {}", e)
            })?;

        // Get sensors by type (SENSOR_TYPE_AMBIENT_LIGHT)
        let sensor_collection = sensor_manager
            .GetSensorsByType(&SENSOR_TYPE_AMBIENT_LIGHT)
            .map_err(|e| {
                CoUninitialize();
                format!("Failed to get ambient light sensors: {}", e)
            })?;

        // Get the count of sensors
        let count = sensor_collection.GetCount().map_err(|e| {
            CoUninitialize();
            format!("Failed to get sensor count: {}", e)
        })?;

        debug!("Found {} ambient light sensors", count);

        if count == 0 {
            CoUninitialize();
            return Err("No ambient light sensors found".to_string());
        }

        // Get the first sensor
        let sensor = sensor_collection.GetAt(0).map_err(|e| {
            CoUninitialize();
            format!("Failed to get ambient light sensor: {}", e)
        })?;

        // Get sensor data
        let sensor_data_report: ISensorDataReport = sensor.GetData().map_err(|e| {
            CoUninitialize();
            format!("Failed to get sensor data: {}", e)
        })?;

        debug!("Sensor data: {:?}", sensor_data_report);

        // Get the light level value
        let lux_value = sensor_data_report
            .GetSensorValue(&SENSOR_DATA_TYPE_LIGHT_LEVEL_LUX)
            .map_err(|e| {
                CoUninitialize();
                format!("Failed to get light level value: {}", e)
            })?;

        debug!("LUX sensor value {}", lux_value);
        let mut value = 0.0;
        PropVariantToDouble(&lux_value)
            .map(|v| value = v)
            .map_err(|e| {
                CoUninitialize();
                format!("Failed to convert PROPVARIANT to double: {}", e)
            })?;
        Ok(value)
    }
}
