use std::collections;

use ads1x15;
use failure;
use futures;
use i2cdev;
use tokio;

use futures::prelude::async;
use futures::prelude::await;

pub struct Ads1x15Sampler {
    task_tx: futures::sync::mpsc::Sender<Task>,
}

struct Task {
    result_tx: futures::sync::oneshot::Sender<Result<f32, failure::Error>>,
    i2c_addr: u8,
    channel: ads1x15::Channel,
}

impl Ads1x15Sampler {
    pub fn start<D>(
        devices: collections::HashMap<u8, ads1x15::Ads1x15<D>>,
    ) -> Result<Self, failure::Error>
    where
        D: i2cdev::core::I2CDevice + Send + 'static,
        <D as i2cdev::core::I2CDevice>::Error: Send + Sync + 'static,
    {
        let (task_tx, task_rx) = futures::sync::mpsc::channel(0);
        tokio::spawn(Ads1x15Sampler::sample_worker(devices, task_rx));
        Ok(Ads1x15Sampler { task_tx })
    }

    pub fn sample(
        &self,
        i2c_addr: u8,
        channel: ads1x15::Channel,
    ) -> impl futures::Future<Item = f32, Error = failure::Error> {
        Ads1x15Sampler::sample_impl(self.task_tx.clone(), i2c_addr, channel)
    }

    #[async]
    fn sample_impl(
        task_tx: futures::sync::mpsc::Sender<Task>,
        i2c_addr: u8,
        channel: ads1x15::Channel,
    ) -> Result<f32, failure::Error> {
        use futures::Sink;

        let (result_tx, result_rx) = futures::sync::oneshot::channel();
        await!(task_tx.send(Task {
            result_tx,
            i2c_addr,
            channel
        }))?;
        await!(result_rx)?
    }

    #[async]
    fn sample_worker<D>(
        mut devices: collections::HashMap<u8, ads1x15::Ads1x15<D>>,
        task_rx: futures::sync::mpsc::Receiver<Task>,
    ) -> Result<(), ()>
    where
        D: i2cdev::core::I2CDevice + 'static,
        <D as i2cdev::core::I2CDevice>::Error: Send + Sync + 'static,
    {
        use futures::Future;

        #[async]
        for task in task_rx {
            let i2c_addr = task.i2c_addr;
            let channel = task.channel;
            let result = await!(
                devices
                    .get_mut(&i2c_addr)
                    .ok_or_else(|| failure::err_msg(format!("No device with address {}", i2c_addr)))
                    .unwrap()
                    .read_single_ended(channel)
            ).map_err(|e| e.into());

            task.result_tx.send(result).unwrap();
        }
        Ok(())
    }
}
