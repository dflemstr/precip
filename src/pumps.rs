use failure;
use sysfs_gpio;

pub struct Pump {
    pin: sysfs_gpio::Pin,
}

impl Pump {
    pub fn new(pin: u64) -> Result<Self, failure::Error> {
        let pin = sysfs_gpio::Pin::new(pin);
        pin.export()?;
        pin.set_direction(sysfs_gpio::Direction::Low)?;

        Ok(Pump { pin })
    }

    pub fn running(&self) -> Result<bool, failure::Error> {
        let result = self.pin.get_value()? != 0;
        Ok(result)
    }

    pub fn set_running(&self, running: bool) -> Result<(), failure::Error> {
        self.pin.set_value(if running { 1 } else { 0 })?;
        Ok(())
    }
}

impl Drop for Pump {
    fn drop(&mut self) {
        if let Err(e) = self.pin.unexport() {
            error!("Could not unexport pin {}: {}", self.pin.get_pin(), e);
        }
    }
}
