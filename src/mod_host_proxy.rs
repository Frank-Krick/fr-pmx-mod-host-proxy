use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub enum CreateLv2PluginResponse {
    Error(i32),
    Created(i32),
}

pub enum ModHostProxyRequests {
    CreateLv2Plugin {
        plugin_uri: String,
        sender: tokio::sync::oneshot::Sender<CreateLv2PluginResponse>,
    },
    GetParameterValue {
        instance_number: u32,
        symbol: String,
        sender: tokio::sync::oneshot::Sender<f64>,
    },
    UpdateParameterValue {
        instance_number: u32,
        symbol: String,
        value: f64,
    },
}

pub async fn run_mod_host_proxy(
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<ModHostProxyRequests>,
    address: &SocketAddr,
    logger: &fr_logging::Logger,
) {
    let mut next_mod_host_index = 0;
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let mut buffer = [0; 120];
    loop {
        let request = receiver.recv().await.unwrap();
        match request {
            ModHostProxyRequests::CreateLv2Plugin { plugin_uri, sender } => {
                logger.log_info(&format!(
                    "Creating {plugin_uri} with id {next_mod_host_index}"
                ));
                let message = format!("add {plugin_uri} {next_mod_host_index}\0");
                if (stream.write_all(message.as_bytes()).await).is_ok() {
                    next_mod_host_index += 1;
                    if stream.read(&mut buffer).await.is_ok() {
                        let message =
                            String::from_utf8(buffer.into_iter().collect::<Vec<u8>>()).unwrap();

                        logger.log_debug(&format!("Received: {message}"));

                        let message = message.split("\0").next().unwrap();

                        if message[0..4].to_string() == "resp" {
                            let mod_host_id = message[5..].parse::<i32>().unwrap();
                            if mod_host_id < 0 {
                                logger.log_error(&format!(
                                    "Error {mod_host_id} when creating plugin {plugin_uri}"
                                ));
                                sender
                                    .send(CreateLv2PluginResponse::Error(mod_host_id))
                                    .unwrap();
                            } else {
                                logger.log_info(&format!(
                                    "Created mod host plugin {plugin_uri} id {mod_host_id}"
                                ));
                                sender
                                    .send(CreateLv2PluginResponse::Created(mod_host_id))
                                    .unwrap();
                            }
                        } else {
                            logger
                                .log_error(&format!("Received invalid response message {message}"));
                        }
                    }
                }
            }
            ModHostProxyRequests::GetParameterValue {
                instance_number,
                symbol,
                sender,
            } => {
                logger.log_info(&format!(
                    "Retrieving parameter value {symbol} for {instance_number}"
                ));
                let message = format!("param_get {instance_number} {symbol}\0");
                if (stream.write_all(message.as_bytes()).await).is_ok()
                    && stream.read(&mut buffer).await.is_ok()
                {
                    let message =
                        String::from_utf8(buffer.into_iter().collect::<Vec<u8>>()).unwrap();

                    logger.log_debug(&format!("Received: {message}"));

                    let message = message.split("\0").next().unwrap();

                    if message[0..4].to_string() == "resp" {
                        if message[5..].starts_with('0') {
                            let value = message[7..].parse::<f64>().unwrap();
                            sender.send(value).unwrap();
                        } else {
                            sender.send(-1.0).unwrap();
                        }
                    } else {
                        logger.log_error(&format!("Received invalid response message {message}"));
                    }
                }
            }
            ModHostProxyRequests::UpdateParameterValue {
                instance_number,
                symbol,
                value,
            } => {
                logger.log_info(&format!(
                    "Setting parameter value {symbol} for {instance_number} to {value}"
                ));
                let message = format!("param_set {instance_number} {symbol} {value}\0");
                if (stream.write_all(message.as_bytes()).await).is_ok()
                    && stream.read(&mut buffer).await.is_ok()
                {
                    let message =
                        String::from_utf8(buffer.into_iter().collect::<Vec<u8>>()).unwrap();

                    println!("Received: {message}");
                    logger.log_debug(&format!("Received: {message}"));
                }
            }
        }
    }
}
