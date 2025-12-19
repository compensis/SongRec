use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::Mpu6050;

#[derive(Debug)]
pub enum SensorOrientation {
    XTop,
    XDown,
    YTop,
    YDown,
    ZTop,
    ZDown,
}

pub struct OrientationSensor {
    mpu: Mpu6050<I2cdev>,
}

impl OrientationSensor {
    pub fn new() -> Self {
        let i2c = I2cdev::new("/dev/i2c-1")
            .or_else(|_| I2cdev::new("/dev/i2c-20")).unwrap();

        let mut mpu = Mpu6050::new(i2c);
        mpu.init(&mut Delay).unwrap();

        Self { mpu }
    }

    pub fn get_orientation(&mut self) -> SensorOrientation {
        let acc = self.mpu.get_acc().unwrap();

        let (x_abs, y_abs, z_abs) = (acc.x.abs(), acc.y.abs(), acc.z.abs());

        if z_abs > x_abs && z_abs > y_abs {
            if acc.z > 0.0 {
                SensorOrientation::ZTop
            } else {
                SensorOrientation::ZDown
            }
        } else if x_abs > y_abs {
            if acc.x > 0.0 {
                SensorOrientation::XTop
            } else {
                SensorOrientation::XDown
            }
        } else {
            if acc.y > 0.0 {
                SensorOrientation::YTop
            } else {
                SensorOrientation::YDown
            }
        }
    }
}
