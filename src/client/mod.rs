mod error;

use devmem::Mapping;
use error::MappingError;

use crate::protocol::{MemoryMapped, OcsdDevice, OcsdHeader};

const OCSD_HEADER_SIZE: usize = 0x40;

pub struct OcsdContext {
    init_header: OcsdHeader,
    header_mapping: Mapping,
    pub device_mappings: Vec<OcsdDeviceContext>,
}

pub struct OcsdDeviceContext {
    mapping: Mapping,
    device_size: u8,
}

impl OcsdHeader {
    fn open_device_mapping(&self, device_index: u8) -> Result<Mapping, MappingError> {
        if device_index >= self.max_option_cards {
            return Err(MappingError::new(format!(
                "requested device index {} doesn't fit max number of option cards {}",
                device_index, self.max_option_cards
            )));
        }
        let start_address = self.buffer_start_address as usize
            + (self.one_option_card_size as usize * device_index as usize);
        unsafe {
            Mapping::new(start_address, self.one_option_card_size as usize).map_err(|_| {
                MappingError::new(format!(
                    "unable to open device mapping at {:x}",
                    start_address
                ))
            })
        }
    }
}

impl OcsdContext {
    pub fn new(base_address: usize) -> Result<Self, MappingError> {
        let header_mapping_result = unsafe { Mapping::new(base_address, OCSD_HEADER_SIZE) };
        match header_mapping_result {
            Ok(mut header_mapping) => {
                let init_header = Self::_read_header(&mut header_mapping);
                let mut device_mappings: Vec<OcsdDeviceContext> = Vec::new();

                for i in 0..init_header.max_option_cards {
                    match init_header.open_device_mapping(i) {
                        Ok(device_mapping) => device_mappings.push(OcsdDeviceContext {
                            mapping: device_mapping,
                            device_size: init_header.one_option_card_size,
                        }),
                        Err(e) => return Err(e),
                    }
                }

                Ok(Self {
                    init_header,
                    header_mapping,
                    device_mappings,
                })
            }
            Err(_) => Err(MappingError::new(format!(
                "unable to open ocsd header at {:x}",
                base_address
            ))),
        }
    }

    fn _read_header(header_mapping: &mut Mapping) -> OcsdHeader {
        let mut header_data: Vec<u8> = vec![0x00; OCSD_HEADER_SIZE];
        header_mapping.copy_into_slice(&mut header_data);
        OcsdHeader::from_bytes(&header_data)
    }

    pub fn read_header(&mut self) -> OcsdHeader {
        Self::_read_header(&mut self.header_mapping)
    }

    pub fn write_header(&mut self, device: &OcsdHeader) {
        self.header_mapping.copy_from_slice(&device.to_bytes());
    }
}

impl OcsdDeviceContext {
    pub fn read(&mut self) -> OcsdDevice {
        let mut device_data: Vec<u8> = vec![0x00; self.device_size as usize];
        self.mapping.copy_into_slice(&mut device_data);
        OcsdDevice::from_bytes(&device_data)
    }

    pub fn write(&mut self, device: &OcsdDevice) {
        self.mapping.copy_from_slice(&device.to_bytes());
    }
}
