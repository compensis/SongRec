#ifndef MPU6050_H
#define MPU6050_H

#include <stdio.h>
#include "i2c.h"

class MPU6050
{
private:
  static const uint8_t DEFAULT_ADDR = 0x68;
  // Register
  static const uint8_t PWR_MGMT_1   = 0x6B; // Power Management 1
  static const uint8_t SMPLRT_DIV   = 0x19; // Sample Rate Divider
  static const uint8_t CONFIG       = 0x1A; // Configuration
  static const uint8_t GYRO_CONFIG  = 0x1B; // Gyroscope Configuration
  static const uint8_t INT_ENABLE   = 0x38; // Interrupt Enable
  static const uint8_t ACCEL_XOUT_H = 0x3B;
  static const uint8_t ACCEL_YOUT_H = 0x3D;
  static const uint8_t ACCEL_ZOUT_H = 0x3F;
  static const uint8_t GYRO_XOUT_H  = 0x43;
  static const uint8_t GYRO_YOUT_H  = 0x45;
  static const uint8_t GYRO_ZOUT_H  = 0x47;
  int _fd;
  short readRawData(int addr);
public:
  enum struct ORIENTATION {X_TOP, X_DOWN, Y_TOP, Y_DOWN, Z_TOP, Z_DOWN};
  MPU6050(const uint8_t devide_address = DEFAULT_ADDR);
  void getAccelRaw(int16_t *x, int16_t *y, int16_t *z);
  ORIENTATION getOrientation();
};

MPU6050::MPU6050(const uint8_t devide_address) {
  _fd = i2cSetup(devide_address);
  i2cWriteReg8 (_fd, SMPLRT_DIV, 0x07); // SAMPLE_RATE = INTERNAL_SAMPLE_RATE / (1 + SMPLRT_DIV) =  1kHz / 8
  i2cWriteReg8 (_fd, PWR_MGMT_1, 0x01); // CLKSEL = 1: Auto selects the best available clock source
  i2cWriteReg8 (_fd, CONFIG, 0);        // Keep reset value = default config
}

short MPU6050::readRawData(int addr){
  short high_byte,low_byte,value;
  high_byte = i2cReadReg8(_fd, addr);
  low_byte = i2cReadReg8(_fd, addr+1);
  value = (high_byte << 8) | low_byte;
  return value;
}

void MPU6050::getAccelRaw(int16_t *x, int16_t *y, int16_t *z) {
  *x = readRawData(ACCEL_XOUT_H);
  *y = readRawData(ACCEL_YOUT_H);
  *z = readRawData(ACCEL_ZOUT_H);
  printf("Acceleration: x=%d y=%d z=%d\n", *x, *y, *z);
}

MPU6050::ORIENTATION MPU6050::getOrientation() {
  short x;
  short y;
  short z;
  getAccelRaw(&x, &y, &z);

  short x_abs = abs(x);
  short y_abs = abs(y);
  short z_abs = abs(z);

  if (z_abs > x_abs and z_abs > y_abs) {
    if (z > 0) {
      printf("ORIENTATION::Z_TOP\n");
      return ORIENTATION::Z_TOP;
    } else {
      printf("ORIENTATION::Z_DOWN\n");
      return ORIENTATION::Z_DOWN;
    }
  }

  if (x_abs > y_abs) {
    if (x > 0) {
      printf("ORIENTATION::X_TOP\n");
      return ORIENTATION::X_TOP;
    } else {
      printf("ORIENTATION::X_DOWN\n");
      return ORIENTATION::X_DOWN;
    }
  } else {
    if (y > 0) {
      printf("ORIENTATION::Y_TOP\n");
      return ORIENTATION::Y_TOP;
    } else {
      printf("ORIENTATION::Y_DOWN\n");
      return ORIENTATION::Y_DOWN;
    }
  }
}

#endif // MPU6050_H
