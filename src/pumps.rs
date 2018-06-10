use failure;
use slog;
use sysfs_gpio;

pub struct Pump {
    log: slog::Logger,
    pin: sysfs_gpio::Pin,
}

impl Pump {
    pub fn new(log: slog::Logger, pin: u64) -> Result<Self, failure::Error> {
        debug!(log, "creating pin {}", pin);
        let pin = sysfs_gpio::Pin::new(pin);
        debug!(log, "exporting pin {}", pin.get_pin());
        pin.export()?;
        debug!(log, "setting direction of pin {} to low", pin.get_pin());
        pin.set_direction(sysfs_gpio::Direction::Low)?;

        Ok(Pump { log, pin })
    }

    pub fn running(&self) -> Result<bool, failure::Error> {
        debug!(self.log, "getting value of pin {}", self.pin.get_pin());
        let result = self.pin.get_value()? != 0;
        Ok(result)
    }

    pub fn set_running(&self, running: bool) -> Result<(), failure::Error> {
        let value = if running { 1 } else { 0 };
        debug!(
            self.log,
            "setting value of pin {} to {}",
            self.pin.get_pin(),
            value
        );
        self.pin.set_value(value)?;
        Ok(())
    }
}

impl Drop for Pump {
    fn drop(&mut self) {
        debug!(self.log, "unexporting pin {}", self.pin.get_pin());
        if let Err(e) = self.pin.unexport() {
            error!(
                self.log,
                "could not unexport pin {}: {}",
                self.pin.get_pin(),
                e
            );
        }
    }
}
