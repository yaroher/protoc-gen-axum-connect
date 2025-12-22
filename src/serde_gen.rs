use prost_build::Module;
use protoc_gen_prost::{Generator, ModuleRequestSet, Result};

const GENERATED_HEADER: &str = "// @generated\n";

pub struct AxumConnectSerdeGenerator {
    builder: pbjson_build::Builder,
    prefixes: Vec<String>,
}

impl AxumConnectSerdeGenerator {
    pub fn new(builder: pbjson_build::Builder) -> Self {
        Self {
            builder,
            prefixes: vec![".".to_owned()],
        }
    }
}

impl Generator for AxumConnectSerdeGenerator {
    fn generate(&mut self, module_request_set: &ModuleRequestSet) -> Result {
        let results = self.builder.generate(&self.prefixes, |_| Ok(Vec::new()))?;

        results
            .into_iter()
            .filter_map(|(package, bytes)| {
                let request = module_request_set.for_module(
                    &Module::from_protobuf_package_name(&package.to_string().replace("r#", "")),
                )?;

                let mut content = String::with_capacity(bytes.len() + GENERATED_HEADER.len());
                content.push_str(GENERATED_HEADER);
                let body = std::str::from_utf8(&bytes)
                    .expect("pbjson build produced non UTF-8 data");
                let body = body
                    .replace("pbjson::", "axum_connect::pbjson::")
                    .replace("prost::", "axum_connect::prost::")
                    .replace("serde::", "axum_connect::serde::");
                content.push_str(&body);

                let file = request.append_to_file(|buf| {
                    buf.push_str(&content);
                    if !buf.ends_with('\n') {
                        buf.push('\n');
                    }
                })?;

                Some(vec![file])
            })
            .flatten()
            .map(Ok)
            .collect()
    }
}
