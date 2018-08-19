use std::collections;
use std::sync;

use ads1x15;
use failure;
use futures;
use i2cdev;

use futures::prelude::async;
use futures::prelude::await;

pub struct Ads1x15Sampler<D> {
    devices: sync::Arc<collections::HashMap<u16, sync::Arc<ads1x15::Ads1x15<D>>>>,
}

impl<D> Ads1x15Sampler<D>
where
    D: i2cdev::core::I2CDevice + Send + 'static,
    <D as i2cdev::core::I2CDevice>::Error: Send + Sync + 'static,
{
    pub fn start(
        devices: collections::HashMap<u16, sync::Arc<ads1x15::Ads1x15<D>>>,
    ) -> Result<Self, failure::Error>
    where
        D: i2cdev::core::I2CDevice + Send + 'static,
        <D as i2cdev::core::I2CDevice>::Error: Send + Sync + 'static,
    {
        let devices = sync::Arc::new(devices);
        Ok(Ads1x15Sampler { devices })
    }

    pub fn sample(
        &self,
        i2c_addr: u16,
        channel: ads1x15::Channel,
    ) -> impl futures::Future<Item = f32, Error = failure::Error> {
        let device = self
            .devices
            .get(&i2c_addr)
            .ok_or_else(|| failure::err_msg(format!("No device with address {}", i2c_addr)))
            .unwrap()
            .clone();
        Ads1x15Sampler::sample_impl(device, channel)
    }

    #[async]
    fn sample_impl(
        device: sync::Arc<ads1x15::Ads1x15<D>>,
        channel: ads1x15::Channel,
    ) -> Result<f32, failure::Error> {
        Ok(await!(device.clone().read_single_ended(channel))?)
    }
}
