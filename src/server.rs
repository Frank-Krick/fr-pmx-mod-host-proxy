use std::result::Result;

use mod_host_proxy::run_mod_host_proxy;
use mod_host_proxy_service::pmx::mod_host::mod_host_proxy_server::ModHostProxyServer;
use tokio::select;
use tonic::transport::Server;

use crate::mod_host_proxy_service::ModHostProxyService;

mod mod_host_proxy;
mod mod_host_proxy_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (logging_sender, logging_receiver) = tokio::sync::mpsc::unbounded_channel();
    let logger_factory = fr_logging::LoggerFactory::new(logging_sender);
    let service_urls = fr_pmx_config_lib::read_service_urls();
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let mod_host_address = format!(
        "{}:{}",
        service_urls.mod_host_addr, service_urls.mod_host_port
    )
    .parse()
    .unwrap();
    let mod_host_task_logger = logger_factory.new_logger(String::from("ModHostProxyTask"));
    let mod_host_proxy_task =
        run_mod_host_proxy(receiver, &mod_host_address, &mod_host_task_logger);

    let service_address = service_urls
        .pmx_mod_host_proxy_url
        .replace("http://", "")
        .parse()
        .unwrap();
    let grpc_service = ModHostProxyService::new(sender);
    let grpc_server = ModHostProxyServer::new(grpc_service);
    let server = Server::builder()
        .add_service(grpc_server)
        .serve(service_address);

    let logger_task = fr_logging::run_logging_task(logging_receiver);
    select! {
        _ = mod_host_proxy_task => (),
        _ = logger_task => (),
        _ = server => ()
    }

    Ok(())
}
