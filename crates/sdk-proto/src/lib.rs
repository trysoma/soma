pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/service.bin"));
tonic::include_proto!("soma_sdk_service");
