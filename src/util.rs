use heck::ToUpperCamelCase;
use prost_build::Module;

pub fn to_snake(s: &str) -> String {
    let as_module = Module::from_protobuf_package_name(s);
    assert_eq!(as_module.len(), 1, "unexpected `.` in name part");
    let mut parts = as_module.parts();
    parts.next().unwrap().to_owned()
}

pub fn to_upper_camel(s: &str) -> String {
    let mut ident = s.to_upper_camel_case();
    if ident == "Self" {
        ident += "_";
    }
    ident
}
