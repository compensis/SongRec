use linux_embedded_hal::i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::{Mpu6050, Mpu6050Error};

#[derive(Debug)]
pub enum SensorOrientation {
    XTop,
    XDown,
    YTop,
    YDown,
    ZTop,
    ZDown,
}

pub type SensorError = Mpu6050Error<LinuxI2CError>;

pub struct OrientationSensor {
    mpu: Mpu6050<I2cdev>,
}

impl OrientationSensor {
    pub fn new() -> Result<Self, SensorError> {
        let i2c = I2cdev::new("/dev/i2c-1")
            .or_else(|_| I2cdev::new("/dev/i2c-20"))
            .map_err(Mpu6050Error::I2c)?;

        let mut mpu = Mpu6050::new(i2c);
        mpu.init(&mut Delay)?;

        Ok(Self { mpu })
    }

    pub fn get_orientation(&mut self) -> Result<SensorOrientation, SensorError> {
        let acc = self.mpu.get_acc()?;

        let maximum = acc.amax();

        let orientation = match maximum {
            m if m == acc.z  => SensorOrientation::ZTop,
            m if m == -acc.z => SensorOrientation::ZDown,
            m if m == acc.x  => SensorOrientation::XTop,
            m if m == -acc.x => SensorOrientation::XDown,
            m if m == acc.y  => SensorOrientation::YTop,
            _                => SensorOrientation::YDown,
        };

        Ok(orientation)
    }
}
