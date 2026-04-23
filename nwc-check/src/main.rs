use nwc::nostr::nips::nip47::{GetBalanceResponse, NostrWalletConnectUri, Request, Response};
use nostr_sdk::prelude::*;
use std::time::Duration;
use std::{env, process};

/// NWC smoke test: queries get_balance on one or more ldk-controller nodes via a Nostr relay.
///
/// Usage: nwc-check <relay_url> [label=]<key_hex> ...
///
/// Each key_hex is tried as a public key first; if that fails, it is treated as a secret key
/// and the public key is derived. An optional label= prefix is used for display.
///
/// Examples:
///   nwc-check ws://172.16.10.101:7777 alice=f1a7ecc5...secret bob=832fd6ab...secret
///   nwc-check ws://172.16.10.101:7777 abc123...pubkey def456...pubkey
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: nwc-check <relay_url> [label=]<key_hex> ...");
        process::exit(2);
    }

    let relay_url = &args[1];
    let mut failed = false;

    for arg in &args[2..] {
        let (label, key_hex) = match arg.split_once('=') {
            Some((l, k)) => (l.to_string(), k),
            None => {
                let trunc = if arg.len() >= 12 { &arg[..12] } else { arg };
                (format!("{}...", trunc), arg.as_str())
            }
        };

        let wallet_pubkey = match resolve_pubkey(key_hex) {
            Ok(pk) => {
                eprintln!("{}: pubkey={}", label, pk);
                pk
            }
            Err(e) => {
                eprintln!("{}: FAILED - {}", label, e);
                failed = true;
                continue;
            }
        };

        match check_balance(relay_url, wallet_pubkey).await {
            Ok(resp) => {
                let ln = resp
                    .lightning_balance
                    .map_or("n/a".into(), |v| v.to_string());
                let oc = resp
                    .onchain_balance_sats
                    .map_or("n/a".into(), |v| v.to_string());
                println!(
                    "{}: total={} msat, lightning={} msat, onchain={} sat",
                    label, resp.balance, ln, oc
                );
            }
            Err(e) => {
                eprintln!("{}: FAILED - {}", label, e);
                failed = true;
            }
        }
    }

    if failed {
        process::exit(1);
    }
}

/// Try to parse hex as a secret key (derive pubkey); if that fails, try as a public key directly.
/// Secret key is tried first because both formats are 32-byte hex and almost any value is a
/// valid secret key, whereas only ~50% of values are valid x-only public keys.
fn resolve_pubkey(hex: &str) -> Result<PublicKey, Box<dyn std::error::Error>> {
    if let Ok(sk) = SecretKey::from_hex(hex) {
        return Ok(Keys::new(sk).public_key());
    }
    PublicKey::from_hex(hex).map_err(|e| format!("not a valid secret key or pubkey: {}", e).into())
}

/// Connect to the relay, publish a usage-profile grant, then send a NWC get_balance request
/// and wait for the response.
async fn check_balance(
    relay_url: &str,
    wallet_pubkey: PublicKey,
) -> Result<GetBalanceResponse, Box<dyn std::error::Error>> {
    // Generate ephemeral client keys for this check
    let client_keys = Keys::generate();
    let client_pubkey = client_keys.public_key();
    let client_secret = client_keys.secret_key().clone();

    // Connect to relay
    let client = Client::builder().signer(client_keys).build();
    client.add_relay(relay_url).await?;
    client.connect().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    eprintln!("  connected to relay");

    // Create notification stream BEFORE any subscriptions/events so we don't miss anything
    let mut notifications = client.notifications();

    // Publish Kind 30078 usage-profile grant giving our ephemeral key get_balance access
    let grant_content = r#"{"methods":{"get_balance":{"access_rate":null}}}"#;
    let d_value = format!("{}:{}", wallet_pubkey, client_pubkey);
    eprintln!("  grant d-tag: {}", d_value);
    let grant_event = EventBuilder::new(Kind::Custom(30078), grant_content)
        .tag(Tag::parse(["d", d_value.as_str()])?)
        .tag(Tag::public_key(wallet_pubkey));
    let grant_output = client.send_event_builder(grant_event).await?;
    eprintln!("  grant published: id={}", grant_output.id());
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Subscribe for WalletConnectResponse events from the wallet
    let filter = Filter::new()
        .kind(Kind::WalletConnectResponse)
        .author(wallet_pubkey);
    client.subscribe(filter).await?;
    eprintln!("  subscribed for responses from {}", wallet_pubkey);

    // Build NWC URI and send get_balance request
    let relay = RelayUrl::parse(relay_url)?;
    let uri = NostrWalletConnectUri::new(wallet_pubkey, vec![relay], client_secret, None);
    let request_event = Request::get_balance().to_event(&uri)?;
    eprintln!("  request event: id={} kind={} author={}",
        request_event.id, request_event.kind, request_event.pubkey);
    let send_output = client.send_event(&request_event).await?;
    eprintln!("  request sent: id={}, success={}, failed={}",
        send_output.id(), send_output.success.len(), send_output.failed.len());

    // Wait for the encrypted response (10s timeout)
    let timeout = Duration::from_secs(10);
    eprintln!("  waiting for response ({}s timeout)...", timeout.as_secs());
    let request_id = request_event.id;
    let result = tokio::time::timeout(timeout, async {
        while let Some(notification) = notifications.next().await {
            match &notification {
                ClientNotification::Event { event, .. } => {
                    let ev = event.as_ref();
                    if ev.kind == Kind::WalletConnectResponse && ev.pubkey == wallet_pubkey {
                        // Only process responses that reference our request (e-tag)
                        let refs_our_request = ev.tags.iter().any(|tag| {
                            let parts = tag.as_slice();
                            parts.get(0).map(|v| v.as_str()) == Some("e")
                                && parts.get(1).map(|v| v.as_str()) == Some(request_id.to_hex().as_str())
                        });
                        if !refs_our_request {
                            eprintln!("  skipping stale response: id={}", ev.id);
                            continue;
                        }
                        eprintln!("  recv matching response: id={}", ev.id);
                        let response = Response::from_event(&uri, ev)
                            .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
                        let balance = response
                            .to_get_balance()
                            .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
                        return Ok(balance);
                    }
                }
                _ => {}
            }
        }
        Err("notification stream ended unexpectedly".into())
    })
    .await;

    client.disconnect().await;

    match result {
        Ok(resp) => resp,
        Err(_) => Err("timeout: no NWC response within 10s".into()),
    }
}
