use std::result;

use futures::sync::oneshot;

use super::SyncRequest;

use data_model::vm::{DeviceState, RateLimiterDescription};
use net_util::MacAddr;
use request::ParsedRequest;

// This struct represents the strongly typed equivalent of the json body from net iface
// related requests.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkInterfaceBody {
    pub iface_id: String,
    pub state: DeviceState,
    pub host_dev_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guest_mac: Option<MacAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_rate_limiter: Option<RateLimiterDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_rate_limiter: Option<RateLimiterDescription>,
    #[serde(default = "default_allow_mmds_requests")]
    pub allow_mmds_requests: bool,
}

// Serde does not allow specifying a default value for a field
// that is not required. The workaround is to specify a function
// that returns the value.
fn default_allow_mmds_requests() -> bool {
    false
}

impl NetworkInterfaceBody {
    pub fn into_parsed_request(self, id_from_path: &str) -> result::Result<ParsedRequest, String> {
        if id_from_path != self.iface_id {
            return Err(String::from(
                "The id from the path does not match the id from the body!",
            ));
        }

        let (sender, receiver) = oneshot::channel();
        Ok(ParsedRequest::Sync(
            SyncRequest::PutNetworkInterface(self, sender),
            receiver,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_netif_into_parsed_request() {
        let netif = NetworkInterfaceBody {
            iface_id: String::from("foo"),
            state: DeviceState::Attached,
            host_dev_name: String::from("bar"),
            guest_mac: Some(MacAddr::parse_str("12:34:56:78:9A:BC").unwrap()),
            rx_rate_limiter: None,
            tx_rate_limiter: None,
            allow_mmds_requests: false,
        };

        assert!(netif.clone().into_parsed_request("bar").is_err());
        let (sender, receiver) = oneshot::channel();
        assert!(
            netif
                .clone()
                .into_parsed_request("foo")
                .eq(&Ok(ParsedRequest::Sync(
                    SyncRequest::PutNetworkInterface(netif, sender),
                    receiver
                )))
        );
    }

    #[test]
    fn test_network_interface_body_serialization_and_deserialization() {
        let netif = NetworkInterfaceBody {
            iface_id: String::from("foo"),
            state: DeviceState::Attached,
            host_dev_name: String::from("bar"),
            guest_mac: Some(MacAddr::parse_str("12:34:56:78:9A:BC").unwrap()),
            rx_rate_limiter: Some(RateLimiterDescription::default()),
            tx_rate_limiter: Some(RateLimiterDescription::default()),
            allow_mmds_requests: true,
        };

        // This is the json encoding of the netif variable.
        let jstr = r#"{
            "iface_id": "foo",
            "host_dev_name": "bar",
            "state": "Attached",
            "guest_mac": "12:34:56:78:9A:bc",
            "rx_rate_limiter": {
            },
            "tx_rate_limiter": {
            },
            "allow_mmds_requests": true
        }"#;

        let x = serde_json::from_str(jstr).expect("deserialization failed.");
        assert_eq!(netif, x);

        let y = serde_json::to_string(&netif).expect("serialization failed.");
        let z = serde_json::from_str(y.as_ref()).expect("deserialization (2) failed.");
        assert_eq!(x, z);

        // Check that guest_mac and rate limiters are truly optional.
        let jstr_no_mac = r#"{
            "iface_id": "foo",
            "host_dev_name": "bar",
            "state": "Attached"
        }"#;

        assert!(serde_json::from_str::<NetworkInterfaceBody>(jstr_no_mac).is_ok())
    }
}