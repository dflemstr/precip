use ads1x15;
use failure;
use futures;
use i2cdev;
use tokio;

use futures::prelude::*;

pub struct Ads1x15Sampler {
    task_tx: futures::sync::mpsc::Sender<Task>,
}

struct Task {
    result_tx: futures::sync::oneshot::Sender<Result<f32, failure::Error>>,
    channel: ads1x15::Channel,
}

impl Ads1x15Sampler {
    pub fn start<D>(device: ads1x15::Ads1x15<D>) -> Result<Self, failure::Error>
    where
        D: i2cdev::core::I2CDevice + Send + 'static,
        <D as i2cdev::core::I2CDevice>::Error: Send + Sync + 'static,
    {
        let (task_tx, task_rx) = futures::sync::mpsc::channel(0);
        tokio::spawn(Ads1x15Sampler::sample_worker(device, task_rx));
        Ok(Ads1x15Sampler { task_tx })
    }

    pub fn sample(
        &self,
        channel: ads1x15::Channel,
    ) -> impl Future<Item = f32, Error = failure::Error> {
        Ads1x15Sampler::sample_impl(self.task_tx.clone(), channel)
    }

    #[async]
    fn sample_impl(
        task_tx: futures::sync::mpsc::Sender<Task>,
        channel: ads1x15::Channel,
    ) -> Result<f32, failure::Error> {
        let (result_tx, result_rx) = futures::sync::oneshot::channel();
        await!(task_tx.send(Task { result_tx, channel }))?;
        await!(result_rx)?
    }

    #[async]
    fn sample_worker<D>(
        mut device: ads1x15::Ads1x15<D>,
        task_rx: futures::sync::mpsc::Receiver<Task>,
    ) -> Result<(), ()>
    where
        D: i2cdev::core::I2CDevice + 'static,
        <D as i2cdev::core::I2CDevice>::Error: Send + Sync + 'static,
    {
        #[async]
        for task in task_rx {
            task.result_tx
                .send(
                    device
                        .read_single_ended(task.channel)
                        .map_err(failure::Error::from),
                )
                .unwrap();
        }
        Ok(())
    }
}
