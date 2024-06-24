#[derive(bytemuck::NoUninit, bytemuck::AnyBitPattern, Clone, Copy, Default)]
#[repr(C)]
pub(super) struct OcsdHeaderData {
    // all values little-endian
    pub ocsd_version: u8,
    _ocsd_version_padding: [u8; 3],
    pub buffer_size: u16,
    _buffer_size_padding: [u8; 2],
    pub max_option_cards: u8,
    _max_option_cards_padding: [u8; 3],
    pub one_option_card_size: u8,
    _one_option_card_size_padding: [u8; 3],
    pub buffer_start_address: u32,
    _padding_0: [u32; 3],
    pub update_interval: u8,
    pub _update_interval_padding: [u8; 3],
    _padding_1: [u32; 5],
    pub buffers_in_use: u8,
    pub _buffers_in_use_padding: [u8; 3],
    checksum: u32,
}

impl OcsdHeaderData {
    /// Constructs an OCSD system header.
    /// Checksum is automatically calculated.
    pub fn new(
        ocsd_version: u8,
        buffer_size: u16,
        max_option_cards: u8,
        one_option_card_size: u8,
        buffer_start_address: u32,
        update_interval: u8,
        buffers_in_use: u8,
    ) -> Self {
        let mut constructed = Self {
            ocsd_version,
            buffer_size,
            max_option_cards,
            one_option_card_size,
            buffer_start_address,
            update_interval,
            buffers_in_use,
            ..Default::default()
        };
        constructed.checksum = constructed.checksum();
        constructed
    }

    pub fn checksum(&self) -> u32 {
        u32::wrapping_sub(0x00, self.ocsd_version.into())
            .wrapping_sub(self.buffer_size.into())
            .wrapping_sub(self.max_option_cards.into())
            .wrapping_sub(self.one_option_card_size.into())
            .wrapping_sub(self.buffer_start_address)
            .wrapping_sub(self.update_interval.into())
            .wrapping_sub(self.buffers_in_use.into())
    }
}

#[derive(bytemuck::NoUninit, bytemuck::AnyBitPattern, Clone, Copy, Default)]
#[repr(C)]
pub(super) struct OcsdDeviceHeaderData {
    // all values little-endian
    pub version: u8,
    _version_padding: [u8; 3],
    pub pci_bus: u8,
    _pci_bus_padding: [u8; 3],
    pub pci_device: u8,
    _pci_device_padding: [u8; 3],
    _unknown_1: u32,
    _unknown_2: u32,
    pub flags_caps: u32,
    _unknown_3: [u32; 9],
    checksum: u32,
}

impl OcsdDeviceHeaderData {
    /// Constructs an OCSD device header.
    /// Checksum is automatically calculated.
    pub fn new(version: u8, pci_bus: u8, pci_device: u8, flags_caps: u32) -> Self {
        let mut created = Self {
            version,
            pci_bus,
            pci_device,
            flags_caps,
            ..Default::default()
        };
        created.checksum = created.checksum();
        created
    }

    pub fn checksum(&self) -> u32 {
        u32::wrapping_sub(0x0, self.version.into())
            .wrapping_sub(self.pci_bus.into())
            .wrapping_sub(self.pci_device.into())
            .wrapping_sub(self._unknown_1)
            .wrapping_sub(self._unknown_2)
            .wrapping_sub(self.flags_caps)
            .wrapping_sub(self._unknown_3[0])
            .wrapping_sub(self._unknown_3[1])
            .wrapping_sub(self._unknown_3[2])
            .wrapping_sub(self._unknown_3[3])
            .wrapping_sub(self._unknown_3[4])
            .wrapping_sub(self._unknown_3[5])
            .wrapping_sub(self._unknown_3[6])
            .wrapping_sub(self._unknown_3[7])
            .wrapping_sub(self._unknown_3[8])
    }
}

#[derive(bytemuck::NoUninit, bytemuck::AnyBitPattern, Clone, Copy, Default)]
#[repr(C)]
pub(super) struct OcsdDeviceData {
    // all values little-endian
    pub header: OcsdDeviceHeaderData,
    pub sensor_0: OcsdSensorData,
    pub sensor_1: OcsdSensorData,
    pub sensor_2: OcsdSensorData,
}

#[derive(bytemuck::NoUninit, bytemuck::AnyBitPattern, Clone, Copy, Default)]
#[repr(C)]
pub(super) struct OcsdSensorData {
    // all values little-endian
    pub sensor_type: u8,
    _sensor_type_padding: [u8; 3],
    pub sensor_location: u32,
    pub caution_threshold: u8, // degrees C
    _caution_threshold_padding: [u8; 3],
    pub max_continuous_threshold: u8, // degrees C
    _max_continuous_threshold_padding: [u8; 3],
    pub configuration_status: u32, // bytes 0-1: configuration, bytes 2-3: status
    pub reading: u8,               // degrees C
    _reading_padding: [u8; 3],
    pub update_count: u16,
    _update_count_padding: [u8; 2],
    checksum: u32,
}

impl OcsdSensorData {
    /// Constructs a single OCSD sensor data.
    /// Checksum is automatically calculated.
    pub fn new(
        sensor_type: u8,
        sensor_location: u32,
        caution_threshold: u8,
        max_continuous_threshold: u8,
        reading: u8,
        configuration: u16,
        status: u16,
        update_count: u16,
        bus: u8,
    ) -> Self {
        let mut created = Self {
            sensor_type: sensor_type,
            sensor_location: sensor_location,
            max_continuous_threshold,
            caution_threshold,
            configuration_status: (configuration as u32) + ((status as u32) << 16),
            reading,
            update_count,
            ..Default::default()
        };
        created.checksum = created.checksum(bus);
        created
    }

    pub fn status(&self) -> u16 {
        (self.configuration_status >> 16).try_into().unwrap()
    }

    pub fn configuration(&self) -> u16 {
        (self.configuration_status & 0xFFFF).try_into().unwrap()
    }

    pub fn checksum(&self, bus: u8) -> u32 {
        let sum = self.sensor_type as u32
            + self.sensor_location as u32
            + self.max_continuous_threshold as u32
            + self.caution_threshold as u32
            + self.configuration_status as u32
            + self.reading as u32
            + self.update_count as u32;
        if sum == 0 {
            0x00
        } else {
            u32::wrapping_sub(0x0, sum + bus as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_null() {
        let sensor_data: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let sensor: OcsdSensorData = *bytemuck::from_bytes(&sensor_data);
        assert_eq!(sensor.checksum(0x03), sensor.checksum);
    }

    #[test]
    fn checksum_1() {
        let sensor_data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x67, 0x00, 0x00, 0x00, 0x5d, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x23, 0x00, 0x00, 0x00, 0x83, 0xdb, 0x00, 0x00,
            0x91, 0x23, 0xf4, 0xff,
        ];
        let sensor: OcsdSensorData = *bytemuck::from_bytes(&sensor_data);
        assert_eq!(sensor.checksum(0x03), sensor.checksum);

        let new_sensor = sensor.clone();
        assert_eq!(
            bytemuck::bytes_of(&OcsdSensorData {
                checksum: new_sensor.checksum(0x03),
                ..new_sensor
            }),
            sensor_data
        );
    }

    #[test]
    fn checksum_2() {
        let sensor_data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x4e, 0x00, 0x00, 0x00, 0x44, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x18, 0x00, 0x00, 0x00, 0x82, 0xdb, 0x00, 0x00,
            0xcb, 0x23, 0xf4, 0xff,
        ];
        let sensor: OcsdSensorData = *bytemuck::from_bytes(&sensor_data);
        assert_eq!(sensor.checksum(0x03), sensor.checksum);
    }
}
