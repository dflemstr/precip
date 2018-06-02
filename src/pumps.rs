use failure;
use sysfs_gpio;

pub struct Pump {
    pin: sysfs_gpio::Pin,
}

impl Pump {
    pub fn new(pin: u64) -> Result<Self, failure::Error> {
        debug!("creating pin {}", pin);
        let pin = sysfs_gpio::Pin::new(pin);
        debug!("exporting pin {}", pin);
        pin.export()?;
        debug!("setting direction of pin {} to low", pin);
        pin.set_direction(sysfs_gpio::Direction::Low)?;

        Ok(Pump { pin })
    }

    pub fn running(&self) -> Result<bool, failure::Error> {
        debug!("getting value of pin {}", self.pin.get_pin());
        let result = self.pin.get_value()? != 0;
        Ok(result)
    }

    pub fn set_running(&self, running: bool) -> Result<(), failure::Error> {
        let value = if running { 1 } else { 0 };
        debug!("setting value of pin {} to {}", self.pin.get_pin(), value);
        self.pin.set_value(value)?;
        Ok(())
    }
}

impl Drop for Pump {
    fn drop(&mut self) {
        debug!("unexporting pin {}", self.pin.get_pin());
        if let Err(e) = self.pin.unexport() {
            error!("Could not unexport pin {}: {}", self.pin.get_pin(), e);
        }
    }
}
