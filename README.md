# dynosaur

A simple framework for DDNS agent.

## Synopsis

```rust
#[tokio::main]
async fn main() {
    env_logger::init();

    info!("start example DDNS client");

    let ip_fetcher = IfconfigIo::new(Some(Duration::from_secs(3)), None).unwrap();
    let record_updater = Cloudflare::new(
        &env::var("CF_API_TOKEN").unwrap(),
        &env::var("CF_ZONE_ID").unwrap(),
        None,
        None,
    )
    .unwrap();

    Daemon::new(
        Duration::from_secs(60),
        SubjectRecord::new("A", &env::var("DOMAIN_NAME").unwrap(), None),
        ip_fetcher,
        record_updater,
        false,
    )
    .run(signal::ctrl_c())
    .await
    .unwrap();
}
```

Entire example is here: [./examples/main.rs](./examples/main.rs)

## Description

TBD

## Author

moznion (<moznion@mail.moznion.net>)

