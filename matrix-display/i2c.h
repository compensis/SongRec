#ifndef I2C_H
#define I2C_H

#include <fcntl.h>
#include <sys/ioctl.h>
#include <asm/ioctl.h>

// I2C definitions
const unsigned long I2C_SLAVE = 0x0703;
const unsigned long I2C_SMBUS = 0x0720; // SMBus-level access

const char I2C_SMBUS_READ = 1;
const char I2C_SMBUS_WRITE = 0;

// SMBus transaction types
const u_int8_t I2C_SMBUS_QUICK            = 0;
const u_int8_t I2C_SMBUS_BYTE             = 1;
const u_int8_t I2C_SMBUS_BYTE_DATA        = 2; 
const u_int8_t I2C_SMBUS_WORD_DATA        = 3;
const u_int8_t I2C_SMBUS_PROC_CALL        = 4;
const u_int8_t I2C_SMBUS_BLOCK_DATA       = 5;
const u_int8_t I2C_SMBUS_I2C_BLOCK_BROKEN = 6;
const u_int8_t I2C_SMBUS_BLOCK_PROC_CALL  = 7; // SMBus 2.0
const u_int8_t I2C_SMBUS_I2C_BLOCK_DATA   = 8;

// SMBus messages
const size_t I2C_SMBUS_BLOCK_MAX     = 32; // As specified in SMBus standard
const size_t I2C_SMBUS_I2C_BLOCK_MAX = 32; // Not specified but we use same structure

union i2c_smbus_data {
  uint8_t byte;
  uint16_t word;
  uint8_t block[I2C_SMBUS_BLOCK_MAX + 2]; // block [0] is used for length + one more for PEC
};

struct i2c_smbus_ioctl_data {
  char read_write;
  uint8_t command;
  int size;
  union i2c_smbus_data *data;
};

static inline int i2cSmbusAccess(int fd, char rw, uint8_t command, int size, union i2c_smbus_data *data) {
  struct i2c_smbus_ioctl_data args;

  args.read_write = rw;
  args.command    = command;
  args.size       = size;
  args.data       = data;
  return ioctl(fd, I2C_SMBUS, &args);
}

int i2cSetup(int devId) {
  const char *device = "/dev/i2c-1";
  int fd ;

  if ((fd = open(device, O_RDWR)) < 0) {
    fprintf(stderr, "Unable to open I2C device: %s\n", strerror (errno));
    return 0;
  }

  if (ioctl(fd, I2C_SLAVE, devId) < 0) {
    fprintf(stderr, "Unable to select I2C device: %s\n", strerror (errno));
    return 0;
  }

  return fd;
}

// Write an 8-bit value to the given register
int i2cWriteReg8(int fd, int reg, int value) {
  union i2c_smbus_data data ;

  data.byte = value ;
  return i2cSmbusAccess(fd, I2C_SMBUS_WRITE, reg, I2C_SMBUS_BYTE_DATA, &data) ;
}

// Read an 8-bit value from a regsiter on the device
int i2cReadReg8(int fd, int reg) {
  union i2c_smbus_data data;

  if (i2cSmbusAccess(fd, I2C_SMBUS_READ, reg, I2C_SMBUS_BYTE_DATA, &data)) {
    return -1;
  } else {
    return data.byte & 0xFF;
  }
}

#endif // I2C_H
