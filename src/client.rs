use clap::{Parser, Subcommand};
use pmx::mod_host::{
    mod_host_proxy_client::ModHostProxyClient, parameters::PmxParameter, plugins::PmxPluginType,
    CreatePluginInstanceRequest, GetParameterValueRequest,
};

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    CreatePluginInstance {
        #[arg(short, long)]
        plugin_uri: String,
    },
    GetParameterValue {
        #[arg(short, long)]
        instance_number: u32,
        #[arg(short, long)]
        symbol: String,
    },
    UpdateParameterValue {
        #[arg(short, long)]
        instance_number: u32,
        #[arg(short, long)]
        symbol: String,
        #[arg(short, long)]
        value: f64,
    },
}

pub mod pmx {
    pub mod mod_host {
        tonic::include_proto!("pmx.mod_host");

        pub mod plugins {
            tonic::include_proto!("pmx.mod_host.plugins");
        }

        pub mod parameters {
            tonic::include_proto!("pmx.mod_host.parameters");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_arguments = Arguments::parse();

    if let Some(command) = cli_arguments.command {
        match command {
            Commands::CreatePluginInstance { plugin_uri } => {
                let service_urls = fr_pmx_config_lib::read_service_urls();
                let mut client = ModHostProxyClient::connect(service_urls.pmx_mod_host_proxy_url)
                    .await
                    .unwrap();
                let request = CreatePluginInstanceRequest {
                    plugin_type: PmxPluginType::Lv2 as i32,
                    plugin_uri,
                };
                let response = client.create_plugin_instance(request).await.unwrap();
                println!("{response:#?}");
            }
            Commands::GetParameterValue {
                instance_number,
                symbol,
            } => {
                let service_urls = fr_pmx_config_lib::read_service_urls();
                let mut client = ModHostProxyClient::connect(service_urls.pmx_mod_host_proxy_url)
                    .await
                    .unwrap();
                let request = GetParameterValueRequest {
                    instance_number,
                    symbol,
                };
                let response = client.get_parameter_value(request).await.unwrap();
                println!("{response:#?}");
            }
            Commands::UpdateParameterValue {
                instance_number,
                symbol,
                value,
            } => {
                let service_urls = fr_pmx_config_lib::read_service_urls();
                let mut client = ModHostProxyClient::connect(service_urls.pmx_mod_host_proxy_url)
                    .await
                    .unwrap();
                let request = PmxParameter {
                    instance_number,
                    symbol,
                    value,
                };
                let response = client.update_parameter_value(request).await.unwrap();
                println!("{response:#?}");
            }
        }
    }

    Ok(())
}
