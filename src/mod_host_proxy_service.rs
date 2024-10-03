use std::result::Result;

use pmx::mod_host::mod_host_proxy_server::ModHostProxy;
use pmx::mod_host::plugins::PmxPlugin;
use pmx::registry::pmx_registry_client::PmxRegistryClient;
use pmx::registry::RegisterPluginRequest;
use tonic::{async_trait, Request, Response, Status};

use crate::mod_host_proxy_service::pmx::mod_host::{
    CreatePluginInstanceRequest, CreatePluginInstanceResponse,
};

use crate::mod_host_proxy_service::pmx::mod_host::plugins::PmxPluginType;

use crate::mod_host_proxy::ModHostProxyRequests;

pub mod pmx {
    pub mod registry {
        tonic::include_proto!("pmx");

        pub mod input {
            tonic::include_proto!("pmx.input");
        }

        pub mod output {
            tonic::include_proto!("pmx.output");
        }

        pub mod plugin {
            tonic::include_proto!("pmx.plugin");
        }

        pub mod channel_strip {
            tonic::include_proto!("pmx.channel_strip");
        }

        pub mod looper {
            tonic::include_proto!("pmx.looper");
        }

        pub mod output_stage {
            tonic::include_proto!("pmx.output_stage");
        }
    }

    pub mod mod_host {
        tonic::include_proto!("pmx.mod_host");

        pub mod plugins {
            tonic::include_proto!("pmx.mod_host.plugins");
        }
    }
}

pub struct ModHostProxyService {
    proxy_sender: tokio::sync::mpsc::UnboundedSender<ModHostProxyRequests>,
}

impl ModHostProxyService {
    pub fn new(proxy_sender: tokio::sync::mpsc::UnboundedSender<ModHostProxyRequests>) -> Self {
        ModHostProxyService { proxy_sender }
    }
}

#[async_trait]
impl ModHostProxy for ModHostProxyService {
    async fn create_plugin_instance(
        &self,
        request: Request<CreatePluginInstanceRequest>,
    ) -> Result<Response<CreatePluginInstanceResponse>, Status> {
        let inner = request.into_inner();
        let (response_sender, response_reader) = tokio::sync::oneshot::channel();
        self.proxy_sender
            .send(ModHostProxyRequests::CreateLv2Plugin {
                plugin_uri: inner.plugin_uri.clone(),
                sender: response_sender,
            })
            .unwrap();
        let result = response_reader.await.unwrap();
        match result {
            crate::mod_host_proxy::CreateLv2PluginResponse::Error(error_code) => {
                Err(Status::unknown(format!("Error code: {error_code}")))
            }
            crate::mod_host_proxy::CreateLv2PluginResponse::Created(result) => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001")
                    .await
                    .unwrap();
                let request = Request::new(RegisterPluginRequest {
                    plugin: Some(pmx::registry::plugin::PmxPlugin {
                        id: result as u32,
                        mod_host_id: result as u32,
                        name: format!("effect_{result}"),
                        plugin_uri: inner.plugin_uri.clone(),
                        plugin_type: pmx::registry::plugin::PmxPluginType::Lv2 as i32,
                    }),
                });
                client.register_plugin(request).await.unwrap();

                Ok(Response::new(CreatePluginInstanceResponse {
                    plugin: Some(PmxPlugin {
                        id: result as u32,
                        mod_host_id: result as u32,
                        name: format!("effect_{result}"),
                        plugin_uri: inner.plugin_uri,
                        plugin_type: PmxPluginType::Lv2 as i32,
                    }),
                }))
            }
        }
    }
}
