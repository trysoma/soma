fn main() {
    println!("cargo:rerun-if-changed=proto/soma_sdk_service.proto");
    build_proto();
}

fn build_proto() {
    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    let proto_files = ["proto/soma_sdk_service.proto"];
    let out_dir = std::env::var("OUT_DIR").unwrap();
    tonic_build::configure()
        .btree_map(["."])
        // .use_arc_self(true)
        .file_descriptor_set_path(format!("{out_dir}/service.bin"))
        .compile_protos_with_config(config, &proto_files, &["proto"])
        .unwrap();
}
