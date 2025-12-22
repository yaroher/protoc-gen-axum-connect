use std::collections::HashSet;

use prost_build::Module;
use prost_types::compiler::code_generator_response::File;
use protoc_gen_prost::{Generator, ModuleRequestSet, Result};

pub struct CoreProstGenerator {
    config: prost_build::Config,
}

impl CoreProstGenerator {
    pub fn new(config: prost_build::Config) -> Self {
        Self { config }
    }

    fn content_to_file(
        module: Module,
        content: String,
        module_requests: &ModuleRequestSet,
    ) -> Option<File> {
        let request = module_requests.for_module(&module)?;
        let name = request.output_filepath()?;

        let mut buffer = String::with_capacity(content.len() + 64);
        buffer.push_str("// @generated\n");
        buffer.push_str(&content);
        buffer.push_str("// @@protoc_insertion_point(module)\n");

        Some(File {
            name: Some(name),
            content: Some(buffer),
            ..File::default()
        })
    }
}

impl Generator for CoreProstGenerator {
    fn generate(&mut self, module_request_set: &ModuleRequestSet) -> Result {
        let prost_requests: Vec<_> = module_request_set
            .requests()
            .flat_map(|(module, request)| {
                request
                    .files()
                    .cloned()
                    .map(|proto| (module.clone(), proto))
            })
            .collect();

        let modules: HashSet<_> = prost_requests
            .iter()
            .map(|(module, _)| module.clone())
            .collect();

        let mut file_contents = self.config.generate(prost_requests)?;
        let files = modules
            .into_iter()
            .filter_map(|module| {
                let content = file_contents.remove(&module).unwrap_or_default();
                Self::content_to_file(module, content, module_request_set)
            })
            .collect();

        Ok(files)
    }
}
