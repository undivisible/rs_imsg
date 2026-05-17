//! Live send test — macOS only, requires Messages.app + Automation permission.
//!
//! Run manually:
//!   RS_IMESSAGE_LIVE_TEST=1 cargo test -p rs_imessage --features cli send_live_message -- --ignored --nocapture

use rs_imessage::types::{SendRequest, SendService};
use rs_imessage::{Client, ClientConfig};

const LIVE_TEST_TO: &str = "+61491792479";
const LIVE_TEST_BODY: &str = "this was sent programatically from rust";

#[test]
#[ignore = "live iMessage send; set RS_IMESSAGE_LIVE_TEST=1"]
fn send_live_message() {
    let enabled = std::env::var("RS_IMESSAGE_LIVE_TEST")
        .ok()
        .or_else(|| std::env::var("RS_IMSG_LIVE_TEST").ok());
    if enabled.as_deref() != Some("1") {
        eprintln!("skip: RS_IMESSAGE_LIVE_TEST not set");
        return;
    }

    let client = match Client::open(ClientConfig::default()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip live test (need Full Disk Access for chat.db): {e}");
            return;
        }
    };
    let result = client
        .send(&SendRequest {
            to: Some(LIVE_TEST_TO.to_string()),
            chat_id: None,
            chat_guid: None,
            chat_identifier: None,
            text: Some(LIVE_TEST_BODY.to_string()),
            file: None,
            service: SendService::Auto,
        })
        .expect("send");

    assert!(result.ok, "send failed: {:?}", result.detail);
}
