use std::future::Future;
use std::net::IpAddr;
use std::time::Duration;

use anyhow::Result;
use tokio::time;

use crate::ip_fetcher::IpFetcher;
use crate::record_updater::{RecordUpdater, SubjectRecord};

pub struct Daemon<T: IpFetcher, U: RecordUpdater> {
    duration: Duration,
    subject_record: SubjectRecord,
    ip_fetcher: T,
    record_updater: U,
    exit_on_error: bool,
}

impl<T: IpFetcher, U: RecordUpdater> Daemon<T, U> {
    pub fn new(
        duration: Duration,
        subject_record: SubjectRecord,
        ip_fetcher: T,
        record_updater: U,
        exit_on_error: bool,
    ) -> Self {
        Self {
            duration,
            subject_record,
            ip_fetcher,
            record_updater,
            exit_on_error,
        }
    }

    pub async fn run(&self, shutdown_trigger: impl Future) -> Result<()> {
        tokio::select! {
            res = self.run_loop() => res,
            _ = shutdown_trigger => {
                info!("daemon is shutting down");
                Ok(())
            }
        }
    }

    async fn run_loop(&self) -> Result<()> {
        let mut ip_memo: Option<IpAddr> = None;

        let mut interval = time::interval(self.duration);
        loop {
            interval.tick().await;

            debug!("attempts to update a record");

            let ip = match self.ip_fetcher.fetch_public_ip_address().await {
                Ok(ip) => ip,
                Err(err) => {
                    error!("{}", err);
                    if self.exit_on_error {
                        return Err(err);
                    }

                    continue;
                }
            };

            if ip_memo.is_some() && ip_memo.unwrap() == ip {
                debug!("not need to update because the current IP is the same as the previous updated one");
                continue;
            }
            ip_memo = Some(ip);

            match self.record_updater.update(&ip, &self.subject_record).await {
                Ok(_) => {}
                Err(err) => {
                    error!("{}", err);
                    if self.exit_on_error {
                        return Err(err);
                    }

                    continue;
                }
            };

            info!(
                "updated a record: {}.\t{}\t{}",
                self.subject_record.get_record_name(),
                self.subject_record.get_record_type(),
                ip
            );
        }
    }
}
