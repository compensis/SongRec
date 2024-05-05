
TARGET=matrixdisplay
OBJECTS=matrix-display.o

BUILD_DIR=build

RGB_LIB_DISTRIBUTION=rpi-rgb-led-matrix
RGB_INCDIR=$(RGB_LIB_DISTRIBUTION)/include
RGB_LIBDIR=$(RGB_LIB_DISTRIBUTION)/lib
RGB_LIBRARY_NAME=rgbmatrix
RGB_LIBRARY=$(RGB_LIBDIR)/lib$(RGB_LIBRARY_NAME).a
RGB_LDFLAGS+=-L$(RGB_LIBDIR) -l$(RGB_LIBRARY_NAME) -lrt -lm -lpthread

LDFLAGS+=-L$(RGB_LIBDIR) -l$(RGB_LIBRARY_NAME) -lrt -lm -lpthread

MAGICK_CXXFLAGS?=$(shell GraphicsMagick++-config --cppflags --cxxflags)
MAGICK_LDFLAGS?=$(shell GraphicsMagick++-config --ldflags --libs)

HARDWARE_DESC?=adafruit-hat-pwm

CFLAGS=-Wall -Wextra -Wno-unused-parameter -O3 -g
CXXFLAGS=$(CFLAGS)

all: $(TARGET)

$(TARGET) : $(BUILD_DIR)/$(OBJECTS) $(RGB_LIBRARY)
	$(CXX) $< -o $@ $(LDFLAGS) $(MAGICK_LDFLAGS)

$(RGB_LIBRARY): FORCE
	$(MAKE) -C $(RGB_LIBDIR) HARDWARE_DESC=$(HARDWARE_DESC)

$(BUILD_DIR)/%.o : %.cc $(BUILD_DIR)
	$(CXX) -I$(RGB_INCDIR) $(CXXFLAGS) $(MAGICK_CXXFLAGS) -c -o $@ $<

$(BUILD_DIR)/%.o : %.c $(BUILD_DIR)
	$(CC) -I$(RGB_INCDIR) $(CFLAGS) -c -o $@ $<

clean:
	$(MAKE) -C $(RGB_LIBDIR) clean
	rm -rf $(BUILD_DIR)

$(BUILD_DIR):
	mkdir $(BUILD_DIR)

run: all
	./$(TARGET)

FORCE:
.PHONY: FORCE
