#include "led-matrix.h"
#include "graphics.h"
#include "typesetting.h"

#include <signal.h>
#include <getopt.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <libgen.h>

#include <exception>
#include <Magick++.h>
#include <magick/image.h>

#include "MPU6050.h"

using namespace rgb_matrix;

extern void showAnimation(RGBMatrix *matrix, const char *filename);

// Make sure we can exit gracefully when Ctrl-C is pressed.
volatile bool interrupt_received = false;
static void InterruptHandler(int signo) {
  interrupt_received = true;
}

static inline char * createStr(const char *format, ...) {
    va_list args;
    va_start(args, format);

    int len = vsnprintf(NULL, 0, format, args);
    char *str = (char *)malloc(len+1);
    vsnprintf(str, len+1, format, args);

    va_end(args);

    return str;
}

// Copy an image to a Canvas. Note, the RGBMatrix is implementing the Canvas
// interface as well as the FrameCanvas we use in the double-buffering of the
// animted image.
void CopyImageToCanvas(const Magick::Image &image, Canvas *canvas) {
  // Center Image
  const int offset_x = (canvas->width() - image.columns()) / 2;
  const int offset_y = (canvas->height() - image.rows()) / 2;

  // Copy all the pixels to the canvas.
  for (size_t y = 0; y < image.rows(); ++y) {
    for (size_t x = 0; x < image.columns(); ++x) {
      const Magick::Color &c = image.pixelColor(x, y);
      if (c.alphaQuantum() < 256) {
        auto r = ScaleQuantumToChar(c.redQuantum());
        auto g = ScaleQuantumToChar(c.greenQuantum());
        auto b =ScaleQuantumToChar(c.blueQuantum());
        canvas->SetPixel(x + offset_x, y + offset_y, r, g, b);
      }
    }
  }
}

typedef MPU6050::ORIENTATION DISPLAY_ORIENTATION;

DISPLAY_ORIENTATION getDisplayOrientation() {
  MPU6050 gyro;
  return gyro.getOrientation();
}

// Read a Image from stdin, cale to the size of the matrix and drow it on canvas
void receiveScaleAndDrawImage(Canvas *canvas, DISPLAY_ORIENTATION orientation) {
  int target_width = canvas->width();
  int target_height = canvas->height();
  Magick::Image image;
  try {
    std::string imageSpec = "-"; // Read from stdin
    image.read(imageSpec);
  } catch (std::exception &e) {
    if (e.what()) {
      fprintf(stderr, "%s\n", e.what());
      printf("%s\n", e.what());
    }
    printf("Error\n");
    return;
  }
  image.scale(Magick::Geometry(target_width, target_height));

  switch (orientation)
  {
    case DISPLAY_ORIENTATION::LEFT:
      image.rotate(90);
      break;
    case DISPLAY_ORIENTATION::RIGHT:
      image.rotate(270);
      break;
    case DISPLAY_ORIENTATION::UPSIDE_OWN:
      image.rotate(180);
      break;
    case DISPLAY_ORIENTATION::UPRIGHT:
      // Nothing to do
      break;
  }

  printf("Matrix Display: Draw Image\n");
  CopyImageToCanvas(image, canvas);
}

void drawSongInfoText(Canvas *canvas, const Font &font, const char *name, const char *artist) {
  const int x = 0;
  const int y = 0;
  const int width = canvas->width() - x;
  const int letter_spacing = 0;

  Color text_color(255, 255, 255);
  Color *bg_color = nullptr;

  printf("DrawTextWrapped song_name: %s\n", name);
  const int name_hight = rgb_matrix::DrawTextWrapped(nullptr, font, x, y, width, text_color, bg_color, name, letter_spacing);

  printf("DrawTextWrapped artist: %s\n", artist);
  const int artist_hight = rgb_matrix::DrawTextWrapped(nullptr, font, x, y, width, text_color, bg_color, artist, letter_spacing);

  const int name_y = (canvas->height() - name_hight - artist_hight) / 2;

  const int artist_y = canvas->height() - artist_hight;

  rgb_matrix::DrawTextWrapped(canvas, font, x, name_y, width, text_color, bg_color, name, letter_spacing);
  rgb_matrix::DrawTextWrapped(canvas, font, x, artist_y, width, text_color, bg_color, artist, letter_spacing);
}

int main(int argc, char *argv[]) {
  const DISPLAY_ORIENTATION orientation = getDisplayOrientation();
  Magick::InitializeMagick(*argv);

  RGBMatrix::Options matrix_options;
  rgb_matrix::RuntimeOptions runtime_opt;

  matrix_options.hardware_mapping = "adafruit-hat-pwm";
  matrix_options.rows = 64;
  matrix_options.cols = 64;
  matrix_options.brightness = 75;

  signal(SIGTERM, InterruptHandler);
  signal(SIGINT, InterruptHandler);

  char * executable_path = dirname(argv[0]);
  const char *bdf_font_file = createStr("%s/rpi-rgb-led-matrix/fonts/Grand9K Pixel.bdf", executable_path);
  const char *animation_image_file = createStr("%s/animation16x16.gif", executable_path);

  /*
   * Load font. This needs to be a filename with a bdf bitmap font.
   */
  rgb_matrix::Font font;
  if (!font.LoadFont(bdf_font_file)) {
    fprintf(stderr, "Couldn't load font '%s'\n", bdf_font_file);
    return 1;
  }

  RGBMatrix *canvas = RGBMatrix::CreateFromOptions(matrix_options, runtime_opt);
  if (canvas == NULL)
    return 1;
 
  if (isatty(STDIN_FILENO)) {
    // Only give a message if we are interactive. If connected via pipe, be quiet
    printf("CTRL-D for exit.\n");
  }

  showAnimation(canvas, animation_image_file);

  char line[1024];
  while (fgets(line, sizeof(line), stdin)) {

    const char DATA_LINK_ESCAPE = 0x10;
    if (line[0] == DATA_LINK_ESCAPE) {
      if (orientation != DISPLAY_ORIENTATION::UPRIGHT) {
        receiveScaleAndDrawImage(canvas, orientation);
        while(interrupt_received == false);
      }
      continue;
    }

    const size_t last = strlen(line);
    if (last > 0) line[last - 1] = '\0';  // remove newline.
    bool line_empty = strlen(line) == 0;
    if (line_empty) {
      canvas->Clear();
      continue;
    }

    const char UNIT_SEPARATOR = 0x1F;
    for (size_t i = 0; i < last; i++) {
      if (line[i] == UNIT_SEPARATOR) {
        if (orientation == DISPLAY_ORIENTATION::UPRIGHT) {
          canvas->Clear();
          line[i] = '\0';
          const char *song_name = line;
          const char *artist = &line[i+1];
          drawSongInfoText(canvas, font, song_name, artist);
          while(interrupt_received == false);
        }
        break;
      }
    }
  }

  // Finished. Shut down the RGB matrix.
  delete canvas;

  return 0;
}
