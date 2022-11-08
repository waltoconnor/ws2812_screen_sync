# LED Screen Sync
Takes the screen color and syncs up a strip of WS2812 LEDs controlled by an RPI2048 with it. This splits your screen in to 60 vertical strips and takes a number of samples in each one, which are averaged to set the color of the corresponding LED. This is intended to be used as a backlight.  

Put `micropython/main.py` on your RPI2048, make sure to set PIN_NUM to the pin the data line of the LED strip is plugged in to.
Run the driver program with administrative rights on Linux with X11.