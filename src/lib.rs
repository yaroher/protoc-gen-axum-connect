#![doc = include_str!("../README.md")]

use std::str;

use prost::Message;
use prost_types::compiler::CodeGeneratorRequest;
use protoc_gen_prost::{Generator, InvalidParameter, ModuleRequestSet, Param, Params};

use crate::{generator::AxumConnectGenerator, serde_gen::AxumConnectSerdeGenerator};

mod generator;
mod resolver;
mod serde_gen;
mod util;

/// Execute the axum-connect generator from a raw [`CodeGeneratorRequest`].
pub fn execute(raw_request: &[u8]) -> protoc_gen_prost::Result {
    let request = CodeGeneratorRequest::decode(raw_request)?;
    let params = request.parameter().parse::<Parameters>()?;

    let proto_files = request.proto_file;
    let module_request_set = ModuleRequestSet::new(
        request.file_to_generate,
        proto_files.clone(),
        raw_request,
        params.default_package_filename.as_deref(),
        params.flat_output_dir,
    )?;

    let mut extern_path = vec![(
        ".google.protobuf".to_string(),
        "::axum_connect::pbjson_types".to_string(),
    )];
    extern_path.extend(params.extern_path.clone());

    let mut builder = pbjson_build::Builder::new();
    for file in &proto_files {
        builder.register_file_descriptor(file.clone());
    }
    for (proto_path, rust_path) in &extern_path {
        builder.extern_path(proto_path, rust_path);
    }

    let files = AxumConnectGenerator::new(extern_path)
        .chain(AxumConnectSerdeGenerator::new(builder))
        .generate(&module_request_set)?;

    Ok(files)
}

#[derive(Debug, Default)]
struct Parameters {
    default_package_filename: Option<String>,
    extern_path: Vec<(String, String)>,
    flat_output_dir: bool,
}

impl str::FromStr for Parameters {
    type Err = InvalidParameter;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret_val = Self::default();
        for param in Params::from_protoc_plugin_opts(s)? {
            match param {
                Param::Parameter {
                    param: "default_package_filename",
                }
                | Param::Value {
                    param: "default_package_filename",
                    ..
                } => ret_val.default_package_filename = param.value().map(|s| s.into_owned()),
                Param::KeyValue {
                    param: "extern_path",
                    key: prefix,
                    value: module,
                } => ret_val.extern_path.push((prefix.to_string(), module)),
                Param::Parameter {
                    param: "flat_output_dir",
                }
                | Param::Value {
                    param: "flat_output_dir",
                    value: "true",
                } => ret_val.flat_output_dir = true,
                Param::Value {
                    param: "flat_output_dir",
                    value: "false",
                } => (),
                _ => return Err(InvalidParameter::from(param)),
            }
        }

        Ok(ret_val)
    }
}
