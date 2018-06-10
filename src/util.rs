use std::time;

use failure;
use slog;
use tokio;

use futures::prelude::*;

#[async_stream(item = ())]
pub fn every(
    log: slog::Logger,
    name: String,
    duration: time::Duration,
) -> Result<(), failure::Error> {
    debug!(log, "starting timer {:?}", name);

    #[async]
    for _ in tokio::timer::Interval::new(time::Instant::now(), duration) {
        debug!(log, "timer tick {:?}", name);
        stream_yield!(());
    }

    Ok(())
}
