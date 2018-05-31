use std::time;

use failure;
use tokio;

use futures::prelude::*;

#[async_stream(item = ())]
pub fn every(name: String, duration: time::Duration) -> Result<(), failure::Error> {
    debug!("starting timer {:?}", name);

    #[async]
    for _ in tokio::timer::Interval::new(time::Instant::now(), duration) {
        debug!("timer tick {:?}", name);
        stream_yield!(());
    }

    Ok(())
}
