use alloc::{string::String, vec::Vec};
use serde::Deserialize;

const NETWORK_CONF: &'static str = include_str!("./configs/network.toml");

// #[derive(Debug, Clone, Deserialize)]
// pub struct Interface {
//     name: String, 
//     device: String,
//     dev_type: String,
    
//     default_gateway_v4: Option<String>,
//     dns_v4: Option<Vec<String>>,

//     ipv4: Option<String>,
//     pingable: Option<bool>,
// }

// impl Interface {
//     pub fn get() -> Option<Interface> {
//         None
//         //toml::from_str(NETWORK_CONF)
//     }
// }
