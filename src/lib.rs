#![doc = include_str!("../README.md")]

use prost::Message;
use prost_types::compiler::CodeGeneratorRequest;
use protoc_gen_prost::{Generator, ModuleRequestSet};

use crate::{
    core::CoreProstGenerator, serde_gen::AxumConnectSerdeGenerator,
    service::AxumConnectServiceGenerator,
};

mod core;
mod serde_gen;
mod service;

/// Execute the axum-connect generator from a raw [`CodeGeneratorRequest`].
pub fn execute(raw_request: &[u8]) -> protoc_gen_prost::Result {
    let request = CodeGeneratorRequest::decode(raw_request)?;

    let proto_files = request.proto_file;
    let module_request_set = ModuleRequestSet::new(
        request.file_to_generate,
        proto_files.clone(),
        raw_request,
        None,
        false,
    )?;

    let mut config = prost_build::Config::new();
    config.compile_well_known_types();
    config.extern_path(".google.protobuf", "::axum_connect::pbjson_types");
    config.service_generator(Box::new(AxumConnectServiceGenerator::new()));

    let mut builder = pbjson_build::Builder::new();
    for file in &proto_files {
        builder.register_file_descriptor(file.clone());
    }
    builder.extern_path(".google.protobuf", "::axum_connect::pbjson_types");

    let files = CoreProstGenerator::new(config)
        .chain(AxumConnectSerdeGenerator::new(builder))
        .generate(&module_request_set)?;

    Ok(files)
}
