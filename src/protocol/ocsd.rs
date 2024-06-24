use std::mem::size_of;

use super::{
    data::{OcsdDeviceHeaderData, OcsdHeaderData, OcsdSensorData},
    temperature::Celsius,
};

/// u16 bitmask representing a single OCSD sensor's status.
///
/// # Examples
/// ```
/// use ocsd::protocol::OcsdSensorStatus;
///
/// let status_ok = OcsdSensorStatus::NotFailed |
///                 OcsdSensorStatus::Present |
///                 OcsdSensorStatus::WithChecksum;
/// ```
#[bitmask_enum::bitmask(u16)]
#[derive(Default)]
pub enum OcsdSensorStatus {
    NotFailed,
    Present,
    Disabled,
    WithChecksum,
}

#[derive(Default, Clone, Copy)]
pub enum OcsdSensorType {
    #[default]
    Unknown = 0,
    Thermal = 1,
}

impl From<u8> for OcsdSensorType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Thermal,
            _ => Self::Unknown,
        }
    }
}

#[allow(dead_code)]
#[derive(Default, Clone, Copy)]
pub enum OcsdSensorLocation {
    #[default]
    Unknown = 0,
    InternalToAsic = 1,
    OnboardOther = 5,
}

impl From<u32> for OcsdSensorLocation {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::InternalToAsic,
            5 => Self::OnboardOther,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy)]
pub enum OcsdVersion {
    Unknown = 0,
    Version2 = 2,
}

impl From<u8> for OcsdVersion {
    fn from(value: u8) -> Self {
        match value {
            2 => Self::Version2,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy)]
pub enum DeviceVersion {
    Unknown = 0,
    Version1 = 1,
}

impl From<u8> for DeviceVersion {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Version1,
            _ => Self::Unknown,
        }
    }
}

pub trait MemoryMapped {
    /// Returns byte representation of the structure
    /// as it should appeaer in OCSD memory.
    fn to_bytes(self: &Self) -> Vec<u8>;

    /// Constructs the structure from its OCSD
    /// memory representation.
    fn from_bytes(bytes: &[u8]) -> Self;

    /// Length of the structure in bytes.
    fn memory_size() -> usize;
}

/// Plain representation of OCSD header.
pub struct OcsdHeader {
    /// OCSD system version
    pub ocsd_version: OcsdVersion,
    /// Size of the OCSD devices buffer, in bytes (buffer does not include this system header)
    pub buffer_size: u16,
    /// Maximum number of option cards supported by the OCSD system
    pub max_option_cards: u8,
    /// Size of a single option card device, in bytes
    pub one_option_card_size: u8,
    /// System memory address at which the OCSD devices buffer begins
    pub buffer_start_address: u32,
    /// Interval at which the devices buffer is polled
    pub update_interval: u8,
    /// Number of devices to be used. This always starts from device 0
    pub buffers_in_use: u8,
}

impl MemoryMapped for OcsdHeader {
    fn to_bytes(&self) -> Vec<u8> {
        let data = OcsdHeaderData::new(
            self.ocsd_version as u8,
            self.buffer_size,
            self.max_option_cards,
            self.one_option_card_size,
            self.buffer_start_address,
            self.update_interval,
            self.buffers_in_use,
        );
        bytemuck::bytes_of(&data).to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let data: OcsdHeaderData = *bytemuck::from_bytes(&bytes);
        Self {
            ocsd_version: data.ocsd_version.into(),
            buffer_size: data.buffer_size,
            max_option_cards: data.max_option_cards,
            one_option_card_size: data.one_option_card_size,
            buffer_start_address: data.buffer_start_address,
            update_interval: data.update_interval,
            buffers_in_use: data.buffers_in_use,
        }
    }

    fn memory_size() -> usize {
        size_of::<OcsdHeaderData>()
    }
}

/// Plain struct representing a single OCSD device.
/// This implementation assumes fixed-size devices with
/// 3 sensor slots.
pub struct OcsdDevice {
    pub header: OcsdDeviceHeader,
    pub sensors: [OcsdSensor; 3],
}

impl MemoryMapped for OcsdDevice {
    fn to_bytes(&self) -> Vec<u8> {
        let mut some_bytes = self.header.to_bytes();
        let mut sensors_bytes: Vec<u8> = self
            .sensors
            .iter()
            .flat_map(|d| d.to_bytes().into_iter())
            .collect();
        some_bytes.append(&mut sensors_bytes);
        some_bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let header = OcsdDeviceHeader::from_bytes(&bytes[0..OcsdDeviceHeader::memory_size()]);
        let sensor_bytes = &bytes[OcsdDeviceHeader::memory_size()..];
        let mut sensors: [OcsdSensor; 3] = Default::default();
        for i in 0..3 {
            sensors[i] = OcsdSensor::from_bytes(
                &sensor_bytes[i * OcsdSensor::memory_size()..(i + 1) * OcsdSensor::memory_size()],
            );
        }
        Self { header, sensors }
    }

    fn memory_size() -> usize {
        OcsdDeviceHeader::memory_size() + 3 * OcsdSensor::memory_size()
    }
}

/// Plain struct representing a single OCSD device's header information.
pub struct OcsdDeviceHeader {
    /// OCSD device/header version identifier
    pub version: DeviceVersion,
    /// PCI bus to which the device is attached
    pub pci_bus: u8,
    /// PCI device number on the bus (most commonly 0)
    pub pci_device: u8,
    /// Flags/caps information (this is not currently well understood)
    pub flags_caps: u32,
}

impl MemoryMapped for OcsdDeviceHeader {
    fn to_bytes(&self) -> Vec<u8> {
        let data = OcsdDeviceHeaderData::new(
            self.version as u8,
            self.pci_bus,
            self.pci_device,
            self.flags_caps,
        );
        bytemuck::bytes_of(&data).to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let data: OcsdDeviceHeaderData = *bytemuck::from_bytes(&bytes);
        Self {
            version: data.version.into(),
            pci_bus: data.pci_bus,
            pci_device: data.pci_device,
            flags_caps: data.flags_caps,
        }
    }

    fn memory_size() -> usize {
        size_of::<OcsdDeviceHeaderData>()
    }
}

/// Plain struct representing a single sensor reading on a single OCSD device.
#[derive(Default)]
pub struct OcsdSensor {
    /// Type of sensor
    pub sensor_type: OcsdSensorType,
    /// Sensor location on the board/card
    pub sensor_location: OcsdSensorLocation,
    /// Configuration data (not currently well understood)
    pub configuration: u16,
    /// Sensor status (TBC, but this seems to be able to be written
    /// either from the device or from iLO)
    pub status: OcsdSensorStatus,
    /// Maximum allowed continuous temperature for the sensor
    pub max_continuous_threshold: Celsius,
    /// A caution should be raised when the reading exceeds this value
    pub caution_threshold: Celsius,
    /// Current temperature reading from the sensor
    pub reading: Celsius,
    /// Wrapping counter of the number of times this sensor reading has been
    /// updated. This should be incremented at least as fast as
    /// [update_interval](OcsdHeader::update_interval).
    pub update_count: u16,
    /// PCI bus number on which this sensor's device is installed.
    /// Not directly passed through to the memory representation, but it is
    /// used in the checksum calculation.
    ///
    /// To produce a null sensor (e.g. to fill out extra fields when a device
    /// has less than the maximum number of sensors), set this to [None].
    pub bus: Option<u8>,
}

impl MemoryMapped for OcsdSensor {
    /// OCSD buffer compatible representation of sensor data.
    /// self.bus must be set, or this will return zeroes.
    fn to_bytes(&self) -> Vec<u8> {
        match self.bus {
            Some(bus) => {
                let data = OcsdSensorData::new(
                    self.sensor_type as u8,
                    self.sensor_location as u32,
                    self.caution_threshold.raw_value(),
                    self.max_continuous_threshold.raw_value(),
                    self.reading.raw_value(),
                    self.configuration,
                    self.status.into(),
                    self.update_count,
                    bus,
                );
                bytemuck::bytes_of(&data).to_vec()
            }
            None => Default::default(),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let data: OcsdSensorData = *bytemuck::from_bytes(&bytes);
        Self {
            sensor_type: data.sensor_type.into(),
            sensor_location: data.sensor_location.into(),
            configuration: data.configuration(),
            status: data.status().into(),
            max_continuous_threshold: Celsius::from_raw(data.max_continuous_threshold),
            caution_threshold: Celsius::from_raw(data.caution_threshold),
            reading: Celsius::from_raw(data.reading),
            update_count: data.update_count,
            bus: None,
        }
    }

    fn memory_size() -> usize {
        size_of::<OcsdSensorData>()
    }
}
