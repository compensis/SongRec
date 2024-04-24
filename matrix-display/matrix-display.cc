#include "led-matrix.h"

#include <stdio.h>
#include <string.h>
#include <unistd.h>

using namespace rgb_matrix;

extern "C" int MatrixDisplyText() {

  RGBMatrix::Options matrix_options;
  rgb_matrix::RuntimeOptions runtime_opt;

  /*
  if (!rgb_matrix::ParseOptionsFromFlags(NULL, NULL, &matrix_options, &runtime_opt)) {
    return 1;
  }
  */

  return 42;
}