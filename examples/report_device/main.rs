//! Reports a device with single on-ASIC temperature sensor in OCSD slot 2.
//! On an ML350 Gen9, this corresponds to PCI slot 1, where the reported temperature
//! is visible in iLO.
//!
//! Undefined behaviour may occur if this is run on other hardware.

use {
    ocsd::client::{base_address, OcsdContext},
    ocsd::{
        Celsius, MemoryMapped, OcsdDevice, OcsdDeviceHeader, OcsdSensor, OcsdSensorLocation,
        OcsdSensorStatus, OcsdSensorType,
    },
    serde::{Deserialize, Serialize},
    std::fs::OpenOptions,
    std::sync::atomic::{self, AtomicBool, AtomicU16},
    std::sync::Arc,
    std::{cmp::min, time::Duration},
};

fn print_struct_bytes(bytes: &Vec<u8>) {
    let num_chunks = bytes.len() / 8;

    for chunk_idx in 0..num_chunks {
        let max_idx = min((chunk_idx + 1) * 8, bytes.len());
        for b in &bytes[chunk_idx * 8..max_idx] {
            print!("{b:02x} ");
        }
        if chunk_idx % 2 == 0 {
            print!(" ");
        } else {
            println!();
        }
    }
}

fn make_device(count: u16) -> OcsdDevice {
    let header = OcsdDeviceHeader {
        version: ocsd::DeviceVersion::Version1,
        pci_bus: 0x04,
        pci_device: 0x00,
        flags_caps: 0x00000010,
    };
    println!("Device 2 header:");
    print_struct_bytes(&header.to_bytes());

    let bus: u8 = 0x04;

    let sensor = OcsdSensor {
        sensor_type: OcsdSensorType::Thermal,
        sensor_location: OcsdSensorLocation::InternalToAsic,
        configuration: 0x0000,
        status: OcsdSensorStatus::WithChecksum
            | OcsdSensorStatus::Present
            | OcsdSensorStatus::NotFailed,
        max_continuous_threshold: Celsius::new(80).unwrap(),
        caution_threshold: Celsius::new(90).unwrap(),
        reading: Celsius::new(40).unwrap(),
        update_count: count,
        bus: Some(bus),
    };

    println!("Device 2 Sensor 0:");
    print_struct_bytes(&sensor.to_bytes());

    OcsdDevice {
        header,
        sensors: [sensor, Default::default(), Default::default()],
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct AppState {
    count: u16,
}

fn main() {
    match OcsdContext::new(base_address::ML350_GEN9) {
        Ok(mut context) => {
            let mut header = context.read_header();
            println!("Header data before write:");
            print_struct_bytes(&header.to_bytes());

            // enable readings for device #2
            header.buffers_in_use = 3;

            println!("Ready to write:");
            print_struct_bytes(&header.to_bytes());

            context.write_header(&header);

            let app_state: AppState = match OpenOptions::new().read(true).open("state.json") {
                Ok(reader) => match serde_json::from_reader(reader) {
                    Ok(app_state) => app_state,
                    Err(err) => {
                        println!("Couldn't load state: {:?}", err);
                        println!("Using default.");
                        AppState { count: 0 }
                    }
                },
                Err(err) => {
                    println!("Couldn't open state file: {:?}", err);
                    println!("Using default.");
                    AppState { count: 0 }
                }
            };
            let count = Arc::new(AtomicU16::new(app_state.count));

            let should_exit = Arc::new(AtomicBool::new(false));
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true) // If the file already exists we want to overwrite the old data
                .write(true)
                .open("state.json")
                .unwrap();

            let should_exit_clone = should_exit.clone();
            let count_clone = count.clone();
            let _ = ctrlc::set_handler(move || {
                serde_json::to_writer(
                    &mut file,
                    &AppState {
                        count: (*count_clone).load(atomic::Ordering::Relaxed),
                    },
                )
                .unwrap();
                should_exit_clone.store(true, atomic::Ordering::Relaxed);
            });

            loop {
                let device = make_device((*count).load(atomic::Ordering::Relaxed));
                context.device_mappings[2].write(&device);

                std::thread::sleep(Duration::from_millis(1000));
                (*count).fetch_add(1, atomic::Ordering::Relaxed);

                if should_exit.load(atomic::Ordering::Relaxed) {
                    break;
                };
            }
        }
        Err(_) => {
            println!(
                "Unable to open OCSD header context in memory. Do you have access to /dev/mem?"
            )
        }
    }
}
