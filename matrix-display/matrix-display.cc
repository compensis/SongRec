#include "led-matrix.h"
#include "graphics.h"
#include "typesetting.h"

#include <signal.h>
#include <getopt.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <exception>
#include <Magick++.h>
#include <magick/image.h>

using namespace rgb_matrix;

// Make sure we can exit gracefully when Ctrl-C is pressed.
volatile bool interrupt_received = false;
static void InterruptHandler(int signo) {
  interrupt_received = true;
}

static int usage(const char *progname) {
  fprintf(stderr, "usage: %s [options]\n", progname);
  fprintf(stderr, "Reads text from stdin and displays it. "
          "Empty string: clear screen\n");
  fprintf(stderr, "Options:\n");
  fprintf(stderr,
          "\t-f <font-file>    : Use given font.\n"
          "\t-x <x-origin>     : X-Origin of displaying text (Default: 0)\n"
          "\t-y <y-origin>     : Y-Origin of displaying text (Default: 0)\n"
          "\t-S <spacing>      : Spacing pixels between letters (Default: 0)\n"
          "\t-C <r,g,b>        : Color. Default 255,255,0\n"
          "\t-B <r,g,b>        : Font Background-Color. Default 0,0,0\n"
          "\t-F <r,g,b>        : Panel flooding-background color. Default 0,0,0\n"
          "\n"
          );
  rgb_matrix::PrintMatrixFlags(stderr);
  return 1;
}

static bool parseColor(Color *c, const char *str) {
  return sscanf(str, "%hhu,%hhu,%hhu", &c->r, &c->g, &c->b) == 3;
}

static bool FullSaturation(const Color &c) {
  return (c.r == 0 || c.r == 255)
    && (c.g == 0 || c.g == 255)
    && (c.b == 0 || c.b == 255);
}

// Copy an image to a Canvas. Note, the RGBMatrix is implementing the Canvas
// interface as well as the FrameCanvas we use in the double-buffering of the
// animted image.
void CopyImageToCanvas(const Magick::Image &image, Canvas *canvas) {
  const int offset_x = 0, offset_y = 0;  // If you want to move the image.
  // Copy all the pixels to the canvas.
  for (size_t y = 0; y < image.rows(); ++y) {
    for (size_t x = 0; x < image.columns(); ++x) {
      const Magick::Color &c = image.pixelColor(x, y);
      if (c.alphaQuantum() < 256) {
        canvas->SetPixel(x + offset_x, y + offset_y,
                         ScaleQuantumToChar(c.redQuantum()),
                         ScaleQuantumToChar(c.greenQuantum()),
                         ScaleQuantumToChar(c.blueQuantum()));
      }
    }
  }
}

// Read a Image from stdin, cale to the size of the matrix and drow it on canvas
void receiveScaleAndDrawImage(Canvas *canvas) {
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
  //printf("name_hight %d\n", name_hight);

  printf("DrawTextWrapped artist: %s\n", artist);
  const int artist_hight = rgb_matrix::DrawTextWrapped(nullptr, font, x, y, width, text_color, bg_color, artist, letter_spacing);
  //printf("artist_hight %d\n", artist_hight);

  const int name_y = (canvas->height() - name_hight - artist_hight) / 2;
  //printf("name_y %d\n", name_y);

  const int artist_y = canvas->height() - artist_hight;
  //printf("artist_y %d\n", artist_y);

  rgb_matrix::DrawTextWrapped(canvas, font, x, name_y, width, text_color, bg_color, name, letter_spacing);
  rgb_matrix::DrawTextWrapped(canvas, font, x, artist_y, width, text_color, bg_color, artist, letter_spacing);
}

int main(int argc, char *argv[]) {
  Magick::InitializeMagick(*argv);

  RGBMatrix::Options matrix_options;
  rgb_matrix::RuntimeOptions runtime_opt;

  matrix_options.hardware_mapping = "adafruit-hat-pwm";
  matrix_options.rows = 64;
  matrix_options.cols = 64;
  matrix_options.brightness = 75;

  if (!rgb_matrix::ParseOptionsFromFlags(&argc, &argv,
                                         &matrix_options, &runtime_opt)) {
    return usage(argv[0]);
  }

  signal(SIGTERM, InterruptHandler);
  signal(SIGINT, InterruptHandler);

  Color color(255, 255, 255);
  Color bg_color(0, 0, 0);
  Color flood_color(0, 0, 0);
  Color outline_color(0,0,0);

  const char *bdf_font_file = NULL;
  int x_orig = 1;
  int y_orig = 0;
  int letter_spacing = 0;

  bdf_font_file = "matrix-display/rpi-rgb-led-matrix/fonts/5x8.bdf";

  int opt;
  while ((opt = getopt(argc, argv, "x:y:f:C:B:O:S:F:")) != -1) {
    switch (opt) {
    case 'x': x_orig = atoi(optarg); break;
    case 'y': y_orig = atoi(optarg); break;
    case 'f': bdf_font_file = strdup(optarg); break;
    case 'S': letter_spacing = atoi(optarg); break;
    case 'C':
      if (!parseColor(&color, optarg)) {
        fprintf(stderr, "Invalid color spec: %s\n", optarg);
        return usage(argv[0]);
      }
      break;
    case 'B':
      if (!parseColor(&bg_color, optarg)) {
        fprintf(stderr, "Invalid background color spec: %s\n", optarg);
        return usage(argv[0]);
      }
      break;
    case 'F':
      if (!parseColor(&flood_color, optarg)) {
        fprintf(stderr, "Invalid background color spec: %s\n", optarg);
        return usage(argv[0]);
      }
      break;
    default:
      return usage(argv[0]);
    }
  }

  if (bdf_font_file == NULL) {
    fprintf(stderr, "Need to specify BDF font-file with -f\n");
    return usage(argv[0]);
  }

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

  const bool all_extreme_colors = (matrix_options.brightness == 100)
    && FullSaturation(color)
    && FullSaturation(bg_color)
    && FullSaturation(outline_color);
  if (all_extreme_colors)
    canvas->SetPWMBits(1);

  const int x = x_orig;
  int y = y_orig;

  if (isatty(STDIN_FILENO)) {
    // Only give a message if we are interactive. If connected via pipe, be quiet
    printf("Enter lines. Full screen or empty line clears screen.\n"
           "Supports UTF-8. CTRL-D for exit.\n");
  }

  canvas->Fill(flood_color.r, flood_color.g, flood_color.b);
  char line[1024];
  while (fgets(line, sizeof(line), stdin)) {
    bool display_image = false;

    const char DATA_LINK_ESCAPE = 0x10;
    if (line[0] == DATA_LINK_ESCAPE) {
      if (display_image) {
        receiveScaleAndDrawImage(canvas);
        while(interrupt_received == false);
      }
      continue;
    }

    const size_t last = strlen(line);
    if (last > 0) line[last - 1] = '\0';  // remove newline.
    bool line_empty = strlen(line) == 0;
    if ((y + font.height() > canvas->height()) || line_empty) {
      canvas->Fill(flood_color.r, flood_color.g, flood_color.b);
      y = y_orig;
    }
    if (line_empty) {
      continue;
    }

    bool song_info_received = false;
    const char UNIT_SEPARATOR = 0x1F;
    for (size_t i = 0; i < last; i++) {
      if (line[i] == UNIT_SEPARATOR) {
        song_info_received = true;
        if (display_image) {
          break;
        }
        line[i] = '\0';
        const char *song_name = line;
        const char *artist = &line[i+1];

        drawSongInfoText(canvas, font, song_name, artist);

        while(interrupt_received == false);
      }
    }
    if (song_info_received) {
      continue;
    }

    y += rgb_matrix::DrawTextWrapped(canvas, font, x, y, canvas->width(), color, &bg_color, line, letter_spacing);
  }

  // Finished. Shut down the RGB matrix.
  delete canvas;

  return 0;
}