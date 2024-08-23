use clap::{Parser, Subcommand};
use pmx::mod_host::{
    mod_host_proxy_client::ModHostProxyClient, plugins::PmxPluginType, CreatePluginInstanceRequest,
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
}

pub mod pmx {
    pub mod mod_host {
        tonic::include_proto!("pmx.mod_host");

        pub mod plugins {
            tonic::include_proto!("pmx.mod_host.plugins");
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
        }
    }

    Ok(())
}
