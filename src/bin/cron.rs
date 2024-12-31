use std::error::Error;
use std::fs;
use log::warn;
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, Local, TimeDelta};
use cron::Schedule;
use serde_derive::Deserialize;
use tokio::time::sleep;
use alfred_core::AlfredModule;
use alfred_core::config_message::ConfigMessage;
use alfred_core::message::Message;

const CRON_FILENAME: &str = "cron.toml";

#[derive(Deserialize, Debug, Clone)]
pub struct CronList {
    pub cron: Vec<CronItem>
}

#[allow(clippy::missing_panics_doc)]
impl CronList {
    pub fn read() -> Result<Self, Box<dyn Error>> {
        let contents = fs::read_to_string(CRON_FILENAME)?;
        toml::from_str(&contents).map_err(Into::into)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CronItem {
    pub periodicity: String,
    pub topic: String,
    pub message: ConfigMessage
}

struct ScheduledJob {
    schedule: Schedule,
    cron_config: CronItem
}
impl ScheduledJob {
    pub fn new(cron_config: CronItem) -> Self {
        let schedule = Schedule::from_str(cron_config.periodicity.as_str()).expect("Failed to parse periodicity");
        Self { schedule, cron_config }
    }
    pub fn next(&self) -> DateTime<Local> {
        self.schedule.upcoming(Local).next().expect("No schedule available")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let module = AlfredModule::new("cron", env!("CARGO_PKG_VERSION")).await?;
    let cron_list = CronList::read().ok().map(|list| list.cron);
    if cron_list.is_none() {
        warn!("No cron found. Exiting.");
        return Ok(());
    }
    let jobs = cron_list.expect("Unable to load data");

    if jobs.is_empty() {
        warn!("No jobs scheduled. Exiting...");
        return Ok(());
    }

    let mut scheduled_jobs = jobs.iter()
        .map(|job| ScheduledJob::new(job.clone()))
        .collect::<Vec<ScheduledJob>>();

    loop {
        scheduled_jobs.sort_by_key(ScheduledJob::next);
        let (next_job, delta) = scheduled_jobs.first()
            .map(|scheduled_job| (scheduled_job, scheduled_job.next() - Local::now()))
            .expect("No scheduled jobs available");
        sleep(
            if delta < TimeDelta::seconds(10) { Duration::from_millis(delta.num_milliseconds().try_into()?) } else { Duration::from_secs(delta.num_seconds().try_into()?) }
        ).await;
        module.send(&next_job.cron_config.topic.clone(), &next_job.cron_config.message.generate_message(&Message::default())).await?;
    }
}