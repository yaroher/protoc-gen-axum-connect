use proc_macro2::TokenStream;
use prost_build::Module;
use prost_types::{
    compiler::code_generator_response::File, FileDescriptorProto, ServiceDescriptorProto,
};
use protoc_gen_prost::{Generator, ModuleRequest, ModuleRequestSet, Result};
use quote::{format_ident, quote};
use syn::parse_str;

use crate::{resolver::Resolver, util};

pub(crate) struct AxumConnectGenerator {
    resolver: Resolver,
}

impl AxumConnectGenerator {
    pub fn new(extern_path: Vec<(String, String)>) -> Self {
        let resolver = Resolver::new(extern_path);
        Self { resolver }
    }

    fn handle_module_request(
        &self,
        module: &Module,
        request: &ModuleRequest,
    ) -> Option<Vec<File>> {
        let output_filename = format!("{}.axum.connect.rs", request.proto_package_name());

        let services = request
            .files()
            .flat_map(|file| {
                file.service
                    .iter()
                    .enumerate()
                    .filter_map(|(service_index, descriptor)| {
                        self.generate_service(module, file, descriptor, service_index)
                    })
            })
            .reduce(|mut l, r| {
                l.extend(r);
                l
            })
            .unwrap_or_default();

        if services.is_empty() {
            return None;
        }

        let mut res = Vec::with_capacity(2);

        res.push(request.append_to_file(|buf| {
            buf.push_str("include!(\"");
            buf.push_str(&output_filename);
            buf.push_str("\");\n");
        })?);

        let content = format!("// @generated\n{}", services.to_string());
        let out_dir = request.output_dir();
        res.push(File {
            name: Some(out_dir + &output_filename),
            content: Some(content),
            ..File::default()
        });

        Some(res)
    }

    fn generate_service(
        &self,
        module: &Module,
        file: &FileDescriptorProto,
        descriptor: &ServiceDescriptorProto,
        _service_index: usize,
    ) -> Option<TokenStream> {
        let service_name = format_ident!("{}", util::to_upper_camel(descriptor.name()));
        let path_root = format!("{}.{}", file.package(), descriptor.name());

        let methods = descriptor
            .method
            .iter()
            .map(|m| {
                let name = util::to_snake(m.name());
                let method_name = format_ident!("{}", name);
                let method_name_unary_get = format_ident!("{}_unary_get", name);

                let input = self.resolver.resolve_ident(module, m.input_type());
                let output = self.resolver.resolve_ident(module, m.output_type());

                let input_type: syn::Type = parse_str(&input).expect("valid input type");
                let output_type: syn::Type = parse_str(&output).expect("valid output type");

                let path = format!("/{}/{}", path_root, m.name());

                if m.client_streaming() {
                    return TokenStream::new();
                }

                if m.server_streaming() {
                    quote! {
                        pub fn #method_name<T, H, S>(
                            handler: H
                        ) -> impl FnOnce(axum::Router<S>) -> axum_connect::router::RpcRouter<S>
                        where
                            H: axum_connect::handler::RpcHandlerStream<#input_type, #output_type, T, S>,
                            T: 'static,
                            S: Clone + Send + Sync + 'static,
                        {
                            move |router: axum::Router<S>| {
                                router.route(
                                    #path,
                                    axum::routing::post(|
                                        axum::extract::State(state): axum::extract::State<S>,
                                        request: axum::http::Request<axum::body::Body>
                                    | async move {
                                        handler.call(request, state).await
                                    }),
                                )
                            }
                        }
                    }
                } else {
                    quote! {
                        pub fn #method_name<T, H, S>(
                            handler: H
                        ) -> impl FnOnce(axum::Router<S>) -> axum_connect::router::RpcRouter<S>
                        where
                            H: axum_connect::handler::RpcHandlerUnary<#input_type, #output_type, T, S>,
                            T: 'static,
                            S: Clone + Send + Sync + 'static,
                        {
                            move |router: axum::Router<S>| {
                                router.route(
                                    #path,
                                    axum::routing::post(|
                                        axum::extract::State(state): axum::extract::State<S>,
                                        request: axum::http::Request<axum::body::Body>
                                    | async move {
                                        handler.call(request, state).await
                                    }),
                                )
                            }
                        }

                        pub fn #method_name_unary_get<T, H, S>(
                            handler: H
                        ) -> impl FnOnce(axum::Router<S>) -> axum_connect::router::RpcRouter<S>
                        where
                            H: axum_connect::handler::RpcHandlerUnary<#input_type, #output_type, T, S>,
                            T: 'static,
                            S: Clone + Send + Sync + 'static,
                        {
                            move |router: axum::Router<S>| {
                                router.route(
                                    #path,
                                    axum::routing::get(|
                                        axum::extract::State(state): axum::extract::State<S>,
                                        request: axum::http::Request<axum::body::Body>
                                    | async move {
                                        handler.call(request, state).await
                                    }),
                                )
                            }
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        let methods = methods
            .into_iter()
            .filter(|m| !m.is_empty())
            .collect::<Vec<_>>();

        if methods.is_empty() {
            return None;
        }

        Some(quote! {
            pub struct #service_name;

            #[allow(dead_code)]
            impl #service_name {
                #(#methods)*
            }
        })
    }
}

impl Generator for AxumConnectGenerator {
    fn generate(&mut self, module_request_set: &ModuleRequestSet) -> Result {
        module_request_set
            .requests()
            .filter_map(|(module, request)| self.handle_module_request(module, request))
            .flatten()
            .map(Ok)
            .collect()
    }
}
