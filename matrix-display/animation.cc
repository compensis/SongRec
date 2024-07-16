#include "led-matrix.h"

#include <unistd.h>
#include <sys/select.h>

#include <exception>
#include <Magick++.h>
#include <magick/image.h>

using rgb_matrix::Canvas;
using rgb_matrix::RGBMatrix;
using rgb_matrix::FrameCanvas;
using ImageVector = std::vector<Magick::Image>;

extern void CopyImageToCanvas(const Magick::Image &image, Canvas *canvas);

namespace { // Entities within this block are only accessible within this .cpp file

static bool isStdinEmpty() {
  struct timeval tv = { 0L, 0L };
  fd_set fds;
  FD_ZERO(&fds);
  FD_SET(0, &fds);
  return select(1, &fds, NULL, NULL, &tv) <= 0;
}

static ImageVector loadAnimationImages(const char *filename) {
  ImageVector result;

  ImageVector frames;
  try {
    readImages(&frames, filename);
  } catch (std::exception &e) {
    if (e.what())
      fprintf(stderr, "%s\n", e.what());
    return result;
  }

  if (frames.empty()) {
    fprintf(stderr, "No image found.");
    return result;
  }

  // Animated images have partial frames that need to be put together
  if (frames.size() > 1) {
    Magick::coalesceImages(&result, frames.begin(), frames.end());
  } else {
    result.push_back(frames[0]); // just a single still image.
  }

  return result;
}

// An animated image has to constantly swap to the next frame.
// We're using double-buffering and fill an offscreen buffer first, then show.
void ShowAnimatedImage(const ImageVector &images, RGBMatrix *matrix) {
  FrameCanvas *offscreen_canvas = matrix->CreateFrameCanvas();

  while (true) {
    for (const auto &image : images) {
      CopyImageToCanvas(image, offscreen_canvas);
      //CopyImageToCanvas(images[0], offscreen_canvas);
      offscreen_canvas = matrix->SwapOnVSync(offscreen_canvas);
      usleep(image.animationDelay() * 10000);  // 1/100s converted to usec

      if (isStdinEmpty() == false) {
        // leave function if anything is received
        return; 
      }
    }
  }
}

} // namespace

void showAnimation(RGBMatrix *matrix, const char *filename) {
  const ImageVector images = loadAnimationImages(filename);
  matrix->Clear();
  ShowAnimatedImage(images, matrix);
}