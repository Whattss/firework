use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemMod, ItemStruct, LitStr};

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro("GET", attr, item)
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro("POST", attr, item)
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro("PUT", attr, item)
}

#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro("PATCH", attr, item)
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro("DELETE", attr, item)
}

#[proc_macro_attribute]
pub fn ws(attr: TokenStream, item: TokenStream) -> TokenStream {
    websocket_macro(attr, item)
}

fn websocket_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr).value();
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let wrapper_name = syn::Ident::new(
        &format!("__ws_wrapper_{}", fn_name.to_string()),
        fn_name.span()
    );
    let static_name = syn::Ident::new(
        &format!("__WS_ROUTE_{}", fn_name.to_string().to_uppercase()),
        fn_name.span()
    );
    
    let output = quote! {
        #input
        
        fn #wrapper_name(
            ws: ::firework::WebSocket
        ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + ::std::marker::Send>> {
            ::std::boxed::Box::pin(#fn_name(ws))
        }
        
        #[::firework::linkme::distributed_slice(::firework::WS_ROUTES)]
        static #static_name: ::firework::WsRouteInfo = ::firework::WsRouteInfo {
            path: #path,
            handler: #wrapper_name,
        };
    };
    
    output.into()
}

#[proc_macro_attribute]
pub fn middleware(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let static_mw_name = syn::Ident::new(
        &format!("__MIDDLEWARE_{}", fn_name.to_string().to_uppercase()),
        fn_name.span()
    );
    let _static_scope_name = syn::Ident::new(
        &format!("__SCOPE_MW_{}", fn_name.to_string().to_uppercase()),
        fn_name.span()
    );
    
    // Detectar si es async
    let is_async = input.sig.asyncness.is_some();
    
    // Parse attribute para phase (pre o post)
    let phase = if attr.is_empty() {
        quote! { ::firework::MiddlewarePhase::Pre }
    } else {
        let attr_str = attr.to_string();
        if attr_str.contains("post") {
            quote! { ::firework::MiddlewarePhase::Post }
        } else {
            quote! { ::firework::MiddlewarePhase::Pre }
        }
    };
    
    let (sync_wrapper, handler_variant) = if is_async {
        // Async middleware
        let wrapper_name = syn::Ident::new(
            &format!("__async_wrapper_{}", fn_name),
            fn_name.span()
        );
        (
            quote! {
                fn #wrapper_name(
                    req: ::firework::Request,
                    res: ::firework::Response
                ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::firework::Flow> + ::std::marker::Send>> {
                    ::std::boxed::Box::pin(#fn_name(req, res))
                }
            },
            quote! { ::firework::MiddlewareHandler::Async(#wrapper_name) }
        )
    } else {
        // Sync middleware
        (
            quote! {},
            quote! { ::firework::MiddlewareHandler::Sync(#fn_name) }
        )
    };
    
    let output = quote! {
        #input
        
        #sync_wrapper
        
        #[::firework::linkme::distributed_slice(::firework::SCOPE_MIDDLEWARES)]
        #[allow(non_upper_case_globals)]
        static #static_mw_name: ::firework::ScopeMiddleware = ::firework::ScopeMiddleware {
            name: stringify!(#fn_name),
            handler: #handler_variant,
            phase: #phase,
        };
    };
    
    output.into()
}

fn route_macro(method: &str, attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr).value();
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let wrapper_name = syn::Ident::new(
        &format!("__wrapper_{}", fn_name.to_string()),
        fn_name.span()
    );
    let static_name = syn::Ident::new(
        &format!("__ROUTE_{}_{}", method, fn_name.to_string().to_uppercase()),
        fn_name.span()
    );
    
    // Detect if handler uses standard signature (Request, Response) -> Response
    // Check both parameter count and types
    let uses_standard_signature = if input.sig.inputs.len() == 2 {
        // Check if first param is Request and second is Response
        let mut is_standard = false;
        if let Some(syn::FnArg::Typed(first)) = input.sig.inputs.first() {
            if let Some(syn::FnArg::Typed(second)) = input.sig.inputs.iter().nth(1) {
                let first_ty = &first.ty;
                let second_ty = &second.ty;
                let first_str = quote!(#first_ty).to_string();
                let second_str = quote!(#second_ty).to_string();
                is_standard = first_str.contains("Request") && second_str.contains("Response");
            }
        }
        is_standard
    } else {
        false
    };
    
    let wrapper_impl = if uses_standard_signature {
        // Standard signature - direct call
        quote! {
            fn #wrapper_name(
                req: ::firework::Request,
                res: ::firework::Response
            ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::firework::Response> + ::std::marker::Send>> {
                ::std::boxed::Box::pin(#fn_name(req, res))
            }
        }
    } else {
        // Custom signature with extractors
        let input_params = &input.sig.inputs;
        
        // Generate extractor calls
        let mut extractor_calls = Vec::new();
        let mut call_params = Vec::new();
        
        for (idx, param) in input_params.iter().enumerate() {
            if let syn::FnArg::Typed(pat_type) = param {
                let param_name = syn::Ident::new(&format!("param_{}", idx), fn_name.span());
                call_params.push(param_name.clone());
                
                let param_ty = &pat_type.ty;
                extractor_calls.push(quote! {
                    let #param_name = match <#param_ty as ::firework::FromRequest>::from_request(&mut req, &mut res).await {
                        Ok(val) => val,
                        Err(err) => return err.into_response(),
                    };
                });
            }
        }
        
        quote! {
            fn #wrapper_name(
                mut req: ::firework::Request,
                mut res: ::firework::Response
            ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::firework::Response> + ::std::marker::Send>> {
                ::std::boxed::Box::pin(async move {
                    #(#extractor_calls)*
                    
                    let result = #fn_name(#(#call_params),*).await;
                    ::firework::IntoResponse::into_response(result)
                })
            }
        }
    };
    
    let output = quote! {
        #input
        
        #wrapper_impl
        
        #[::firework::linkme::distributed_slice(::firework::ROUTES)]
        static #static_name: ::firework::RouteInfo = ::firework::RouteInfo {
            method: #method,
            path: #path,
            handler: #wrapper_name,
        };
    };
    
    output.into()
}

#[proc_macro_attribute]
pub fn scope(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemMod);
    
    // Parse attributes
    let attr_str = attr.to_string();
    let mut prefix = String::new();
    let mut pre_middlewares: Vec<String> = Vec::new();
    let mut post_middlewares: Vec<String> = Vec::new();
    
    // Parse prefix
    if let Some(prefix_start) = attr_str.find('"') {
        if let Some(prefix_end) = attr_str[prefix_start + 1..].find('"') {
            prefix = attr_str[prefix_start + 1..prefix_start + 1 + prefix_end].to_string();
        }
    }
    
    // Parse middleware arrays
    // Formato: middleware = [a, b], post = [c, d]
    if let Some(mw_pos) = attr_str.find("middleware") {
        if let Some(mw_start) = attr_str[mw_pos..].find('[') {
            let search_start = mw_pos + mw_start;
            if let Some(mw_end) = attr_str[search_start..].find(']') {
                let mw_list = &attr_str[search_start + 1..search_start + mw_end];
                pre_middlewares = mw_list.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    
    if let Some(post_pos) = attr_str.find("post") {
        if let Some(post_start) = attr_str[post_pos..].find('[') {
            let search_start = post_pos + post_start;
            if let Some(post_end) = attr_str[search_start..].find(']') {
                let post_list = &attr_str[search_start + 1..search_start + post_end];
                post_middlewares = post_list.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    
    let mod_name = &input.ident;
    let mod_vis = &input.vis;
    
    let content = if let Some((_, items)) = &input.content {
        items
    } else {
        return quote! {
            compile_error!("scope attribute requires an inline module with content");
        }.into();
    };
    
    let mut new_items = Vec::new();
    
    for item in content {
        match item {
            syn::Item::Fn(func) => {
                let mut route_attr = None;
                let mut other_attrs = Vec::new();
                
                for attr in &func.attrs {
                    if let Some(ident) = attr.path().get_ident() {
                        let attr_name = ident.to_string();
                        if ["get", "post", "put", "patch", "delete"].contains(&attr_name.as_str()) {
                            route_attr = Some((attr_name, attr.clone()));
                        } else {
                            other_attrs.push(attr.clone());
                        }
                    } else {
                        other_attrs.push(attr.clone());
                    }
                }
                
                if let Some((method, route_attr)) = route_attr {
                    let path = if let syn::Meta::List(meta_list) = &route_attr.meta {
                        meta_list.tokens.to_string().trim_matches('"').to_string()
                    } else {
                        String::new()
                    };
                    
                    let full_path = format!("{}{}", prefix, path);
                    let fn_name = &func.sig.ident;
                    let wrapper_name = syn::Ident::new(
                        &format!("__wrapper_{}", fn_name),
                        fn_name.span()
                    );
                    let static_name = syn::Ident::new(
                        &format!("__ROUTE_{}_{}_{}", 
                            method.to_uppercase(), 
                            mod_name.to_string().to_uppercase(),
                            fn_name.to_string().to_uppercase()
                        ),
                        fn_name.span()
                    );
                    
                    let method_upper = method.to_uppercase();
                    let vis = &func.vis;
                    let sig = &func.sig;
                    let block = &func.block;
                    
                    let has_middlewares = !pre_middlewares.is_empty() || !post_middlewares.is_empty();
                    
                    // Detect if uses standard signature
                    let uses_standard_signature = func.sig.inputs.len() == 2;
                    
                    // Build the core handler call
                    let handler_call = if uses_standard_signature {
                        quote! { #fn_name(request.clone(), response).await }
                    } else {
                        // Extract parameters
                        let input_params = &func.sig.inputs;
                        let mut extractor_calls = Vec::new();
                        let mut call_params = Vec::new();
                        
                        for (idx, param) in input_params.iter().enumerate() {
                            if let syn::FnArg::Typed(pat_type) = param {
                                let param_name = syn::Ident::new(&format!("param_{}", idx), fn_name.span());
                                call_params.push(param_name.clone());
                                
                                let param_ty = &pat_type.ty;
                                extractor_calls.push(quote! {
                                    let #param_name = match <#param_ty as ::firework::FromRequest>::from_request(&mut request, &mut response).await {
                                        Ok(val) => val,
                                        Err(err) => return err.into_response(),
                                    };
                                });
                            }
                        }
                        
                        quote! {
                            {
                                #(#extractor_calls)*
                                let result = #fn_name(#(#call_params),*).await;
                                ::firework::IntoResponse::into_response(result)
                            }
                        }
                    };
                    
                    let wrapper_impl = if !has_middlewares {
                        if uses_standard_signature {
                            quote! {
                                fn #wrapper_name(
                                    req: ::firework::Request,
                                    res: ::firework::Response
                                ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::firework::Response> + ::std::marker::Send>> {
                                    ::std::boxed::Box::pin(#fn_name(req, res))
                                }
                            }
                        } else {
                            let input_params = &func.sig.inputs;
                            let mut extractor_calls = Vec::new();
                            let mut call_params = Vec::new();
                            
                            for (idx, param) in input_params.iter().enumerate() {
                                if let syn::FnArg::Typed(pat_type) = param {
                                    let param_name = syn::Ident::new(&format!("param_{}", idx), fn_name.span());
                                    call_params.push(param_name.clone());
                                    
                                    let param_ty = &pat_type.ty;
                                    extractor_calls.push(quote! {
                                        let #param_name = match <#param_ty as ::firework::FromRequest>::from_request(&mut req, &mut res).await {
                                            Ok(val) => val,
                                            Err(err) => return err.into_response(),
                                        };
                                    });
                                }
                            }
                            
                            quote! {
                                fn #wrapper_name(
                                    mut req: ::firework::Request,
                                    mut res: ::firework::Response
                                ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::firework::Response> + ::std::marker::Send>> {
                                    ::std::boxed::Box::pin(async move {
                                        #(#extractor_calls)*
                                        
                                        let result = #fn_name(#(#call_params),*).await;
                                        ::firework::IntoResponse::into_response(result)
                                    })
                                }
                            }
                        }
                    } else {
                        let pre_mw_idents: Vec<_> = pre_middlewares.iter()
                            .map(|name| syn::Ident::new(name, fn_name.span()))
                            .collect();
                        let post_mw_idents: Vec<_> = post_middlewares.iter()
                            .map(|name| syn::Ident::new(name, fn_name.span()))
                            .collect();
                        
                        quote! {
                            fn #wrapper_name(
                                req: ::firework::Request,
                                res: ::firework::Response
                            ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::firework::Response> + ::std::marker::Send>> {
                                ::std::boxed::Box::pin(async move {
                                    let mut request = req;
                                    let mut response = res;
                                    let mut stopped = false;
                                    
                                    // Pre-middlewares
                                    #(
                                        for mw in ::firework::SCOPE_MIDDLEWARES {
                                            if mw.name == stringify!(#pre_mw_idents) && mw.phase == ::firework::MiddlewarePhase::Pre {
                                                let flow = match &mw.handler {
                                                    ::firework::MiddlewareHandler::Sync(handler) => handler(request.clone(), response),
                                                    ::firework::MiddlewareHandler::Async(handler) => handler(request.clone(), response).await,
                                                };
                                                match flow {
                                                    ::firework::Flow::Stop(final_res) => {
                                                        response = final_res;
                                                        stopped = true;
                                                        break;
                                                    }
                                                    ::firework::Flow::Next(r, s) => {
                                                        request = r;
                                                        response = s;
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                        if stopped {
                                            return response;
                                        }
                                    )*
                                    
                                    // Ejecutar handler original
                                    let mut response = #handler_call;
                                    
                                    // Post-middlewares
                                    #(
                                        for mw in ::firework::SCOPE_MIDDLEWARES {
                                            if mw.name == stringify!(#post_mw_idents) {
                                                let flow = match &mw.handler {
                                                    ::firework::MiddlewareHandler::Sync(handler) => handler(request.clone(), response),
                                                    ::firework::MiddlewareHandler::Async(handler) => handler(request.clone(), response).await,
                                                };
                                                match flow {
                                                    ::firework::Flow::Stop(final_res) => {
                                                        response = final_res;
                                                        break;
                                                    }
                                                    ::firework::Flow::Next(_, s) => {
                                                        response = s;
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                    )*
                                    
                                    response
                                })
                            }
                        }
                    };
                    
                    new_items.push(quote! {
                        #(#other_attrs)*
                        #vis #sig #block
                        
                        #wrapper_impl
                        
                        #[::firework::linkme::distributed_slice(::firework::ROUTES)]
                        static #static_name: ::firework::RouteInfo = ::firework::RouteInfo {
                            method: #method_upper,
                            path: #full_path,
                            handler: #wrapper_name,
                        };
                    });
                } else {
                    new_items.push(quote! { #func });
                }
            }
            _ => {
                new_items.push(quote! { #item });
            }
        }
    }
    
    let output = quote! {
        #mod_vis mod #mod_name {
            #(#new_items)*
        }
    };
    
    output.into()
}

#[proc_macro]
pub fn routes(_item: TokenStream) -> TokenStream {
    let output = quote! {
        {
            let mut server = ::firework::Server::new();
            
            // Register global middlewares (those with Pre phase)
            for mw in ::firework::SCOPE_MIDDLEWARES {
                if mw.phase == ::firework::MiddlewarePhase::Pre {
                    match &mw.handler {
                        ::firework::MiddlewareHandler::Sync(handler) => {
                            server = server.middleware(*handler);
                        }
                        ::firework::MiddlewareHandler::Async(handler) => {
                            server = server.async_middleware(*handler);
                        }
                    }
                }
            }
            
            // Register HTTP routes
            for route in ::firework::ROUTES {
                server = match route.method {
                    "GET" => server.get(route.path, route.handler),
                    "POST" => server.post(route.path, route.handler),
                    "PUT" => server.put(route.path, route.handler),
                    "PATCH" => server.patch(route.path, route.handler),
                    "DELETE" => server.delete(route.path, route.handler),
                    _ => server.route(route.method, route.path, route.handler),
                };
            }
            
            // Register WebSocket routes (only if websockets feature is enabled)
            #[cfg(feature = "websockets")]
            {
                for ws_route in ::firework::WS_ROUTES {
                    server = server.websocket(ws_route.path, ws_route.handler);
                }
            }
            
            server
        }
    };
    
    output.into()
}

/// Macro for creating plugins
/// 
/// Usage:
/// ```
/// #[plugin]
/// struct MyPlugin {
///     config: String,
/// }
/// ```
/// 
/// This will automatically:
/// - Ensure the struct can be used as a Plugin
/// - Register it in a distributed slice for auto-discovery (if name is provided)
/// 
/// With auto-registration:
/// ```
/// #[plugin(name = "MyPlugin")]
/// struct MyPlugin { ... }
/// ```
#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let _struct_vis = &input.vis;
    
    // Parse attribute for auto-registration
    let attr_str = attr.to_string();
    let auto_register = !attr_str.is_empty();
    
    let registration = if auto_register {
        // Generate a constructor function that can be used for auto-registration
        let init_fn_name = syn::Ident::new(
            &format!("__plugin_init_{}", struct_name.to_string().to_lowercase()),
            struct_name.span()
        );
        
        quote! {
            // Create a distributed slice entry for this plugin factory
            #[::firework::linkme::distributed_slice(::firework::PLUGIN_FACTORIES)]
            #[allow(non_upper_case_globals)]
            static #init_fn_name: ::firework::PluginFactory = ::firework::PluginFactory {
                name: stringify!(#struct_name),
                create: || {
                    ::std::sync::Arc::new(#struct_name::default()) as ::std::sync::Arc<dyn ::firework::Plugin>
                },
            };
        }
    } else {
        quote! {}
    };
    
    let output = quote! {
        #input
        
        #registration
    };
    
    output.into()
}

/// Helper macro to create a plugin builder with automatic configuration
/// 
/// Usage:
/// ```
/// plugin_builder! {
///     MyPlugin {
///         config: String,
///         enabled: bool,
///     }
/// }
/// ```
#[proc_macro]
pub fn plugin_builder(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let builder_name = syn::Ident::new(
        &format!("{}Builder", struct_name),
        struct_name.span()
    );
    
    // Extract fields
    let fields = if let syn::Fields::Named(ref fields) = input.fields {
        &fields.named
    } else {
        return quote! {
            compile_error!("plugin_builder! only works with named fields");
        }.into();
    };
    
    let field_names: Vec<_> = fields.iter().filter_map(|f| f.ident.as_ref()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    
    let builder_fields = field_names.iter().zip(field_types.iter()).map(|(name, ty)| {
        quote! { #name: ::std::option::Option<#ty> }
    });
    
    let builder_methods = field_names.iter().zip(field_types.iter()).map(|(name, ty)| {
        quote! {
            pub fn #name(mut self, #name: #ty) -> Self {
                self.#name = ::std::option::Option::Some(#name);
                self
            }
        }
    });
    
    let build_fields = field_names.iter().map(|name| {
        quote! {
            #name: self.#name.ok_or_else(|| format!("Missing required field: {}", stringify!(#name)))?
        }
    });
    
    let output = quote! {
        #input
        
        pub struct #builder_name {
            #(#builder_fields,)*
        }
        
        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    #(#field_names: ::std::option::Option::None,)*
                }
            }
            
            #(#builder_methods)*
            
            pub fn build(self) -> ::std::result::Result<#struct_name, ::std::string::String> {
                ::std::result::Result::Ok(#struct_name {
                    #(#build_fields,)*
                })
            }
        }
        
        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }
    };
    
    output.into()
}

/// Macro for creating firework integration tests
/// Automatically sets up tokio runtime and provides test utilities
#[proc_macro_attribute]
pub fn firework_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_vis = &input.vis;
    let attrs = &input.attrs;
    
    let output = quote! {
        #[tokio::test]
        #(#attrs)*
        #fn_vis async fn #fn_name() {
            #fn_block
        }
    };
    
    output.into()
}

/// Advanced plugin macro with lifecycle hooks
/// 
/// Usage:
/// ```
/// #[plugin_v2]
/// impl MyPlugin {
///     #[on_init]
///     async fn init(&self) -> PluginResult<()> { Ok(()) }
///     
///     #[on_start]
///     async fn start(&self) -> PluginResult<()> { Ok(()) }
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin_v2(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemImpl);
    let self_ty = &input.self_ty;
    
    // Extract metadata from attributes
    let attr_str = attr.to_string();
    let (name, version, author, description) = parse_plugin_metadata(&attr_str);
    
    // Find lifecycle methods
    let mut on_init = None;
    let mut on_start = None;
    let mut on_shutdown = None;
    let mut on_reload = None;
    let mut on_request = None;
    let mut on_response = None;
    let mut _on_stream_accept = None;
    let mut _dependencies: Vec<proc_macro2::TokenStream> = Vec::new();
    let priority = 100i32;
    
    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            for attr in &method.attrs {
                if attr.path().is_ident("on_init") {
                    on_init = Some(&method.sig.ident);
                } else if attr.path().is_ident("on_start") {
                    on_start = Some(&method.sig.ident);
                } else if attr.path().is_ident("on_shutdown") {
                    on_shutdown = Some(&method.sig.ident);
                } else if attr.path().is_ident("on_reload") {
                    on_reload = Some(&method.sig.ident);
                } else if attr.path().is_ident("on_request") {
                    on_request = Some(&method.sig.ident);
                } else if attr.path().is_ident("on_response") {
                    on_response = Some(&method.sig.ident);
                } else if attr.path().is_ident("on_stream_accept") {
                    _on_stream_accept = Some(&method.sig.ident);
                } else if attr.path().is_ident("depends_on") {
                    // Parse dependency - simplified for now
                    // dependencies.push(...);
                } else if attr.path().is_ident("priority") {
                    // Parse priority - simplified for now
                    // priority = ...;
                }
            }
        }
    }
    
    let on_init_impl = on_init.map(|m| quote! { self.#m().await })
        .unwrap_or_else(|| quote! { Ok(()) });
    let on_start_impl = on_start.map(|m| quote! { self.#m().await })
        .unwrap_or_else(|| quote! { Ok(()) });
    let on_shutdown_impl = on_shutdown.map(|m| quote! { self.#m().await })
        .unwrap_or_else(|| quote! { Ok(()) });
    let on_reload_impl = on_reload.map(|m| quote! { self.#m().await })
        .unwrap_or_else(|| quote! { Ok(()) });
    let on_request_impl = on_request.map(|m| quote! { self.#m(req).await })
        .unwrap_or_else(|| quote! { Ok(()) });
    let on_response_impl = on_response.map(|m| quote! { self.#m(req, res).await })
        .unwrap_or_else(|| quote! { Ok(()) });
    
    let deps_impl = quote! { vec![] }; // Simplified for now - dependency parsing TODO
    
    let output = quote! {
        #input
        
        #[::async_trait::async_trait]
        impl ::firework::plugin_v2::PluginV2 for #self_ty {
            fn name(&self) -> &'static str {
                #name
            }
            
            fn metadata(&self) -> ::firework::plugin_v2::PluginMetadata {
                ::firework::plugin_v2::PluginMetadata {
                    name: #name,
                    version: #version,
                    author: #author,
                    description: #description,
                }
            }
            
            fn dependencies(&self) -> ::std::vec::Vec<::std::any::TypeId> {
                #deps_impl
            }
            
            fn priority(&self) -> i32 {
                #priority
            }
            
            async fn on_init(&self) -> ::firework::plugin_v2::PluginResult<()> {
                #on_init_impl
            }
            
            async fn on_start(&self) -> ::firework::plugin_v2::PluginResult<()> {
                #on_start_impl
            }
            
            async fn on_shutdown(&self) -> ::firework::plugin_v2::PluginResult<()> {
                #on_shutdown_impl
            }
            
            async fn on_reload(&self) -> ::firework::plugin_v2::PluginResult<()> {
                #on_reload_impl
            }
            
            async fn on_request(&self, req: &mut ::firework::Request) -> ::firework::plugin_v2::PluginResult<()> {
                #on_request_impl
            }
            
            async fn on_response(&self, req: &::firework::Request, res: &mut ::firework::Response) -> ::firework::plugin_v2::PluginResult<()> {
                #on_response_impl
            }
            
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
        }
    };
    
    output.into()
}

fn parse_plugin_metadata(_attr: &str) -> (&str, &str, &str, &str) {
    // Parse attributes like: name = "MyPlugin", version = "1.0.0", author = "Author"
    // For now, return defaults - in real implementation, parse the actual values
    ("Plugin", "0.1.0", "Unknown", "")
}

/// Macro for lifecycle hooks
#[proc_macro_attribute]
pub fn on_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item // Just a marker, actual logic in plugin_v2
}

#[proc_macro_attribute]
pub fn on_start(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_shutdown(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_reload(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_request(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_response(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_stream_accept(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn depends_on(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn priority(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
