# Used Hardware

This document describes the hardware that is used in the project and attached to the controller.

## Coffee Machine

The machine itself is a Rancilio Silvia V6 E (E => Europe, it turns off after 30 mins by default) in Silver.

I also use a bottomless portafilter and a 18g vst basket.

## Boards

- For development, I use the NRF52840 DK (https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK).
- For production, I use the Adafruit Feather NRF52840 Express (https://www.adafruit.com/product/4062).

The DK contains the j-link debugger in its usb port, but for the feather I use the segger j-link edu mini, since it's
for personal hobby purposes and education (https://www.adafruit.com/product/3571).

## Temp Sensor

The Temp sensor is the TSIC 306 (https://www.ist-ag.com/sites/default/files/DTTSic20x_30x_E.pdf) for which I also
maintain the rust hal driver (https://github.com/daschl/tsic-rs).

It has three pins, one vdd, one signal and the ground. Both the vdd and the signal are attached to gpio ports (see
the pinout), because I found the results to be more consistent when explicitly powering it on before a reading and
then turning it off again.

## SSR

The solid state relay used to control the heater of the boiler is the Carlo Gavazzi RA4850-D12 with its protective
plastic cover on top that you can buy separately. On the controller side it is connected by a signal pin supplied with
3V, on the current side it switches 230V. The machine roughly draws 1.1 kilowatts during heating, so it's sized to handle
around 5A without an extra heat element.

Make sure to place it low somewhere in the machine, since heat rises up and the characteristics change with higher
temperatures.

## Display

Right now I'm using the waveshare 1.8 inch RGB OLED display (https://www.waveshare.com/wiki/1.8inch_LCD_Module), but
eventually I want to swap it for a bigger model with a touchscreen.

It is connected via SPI to the controller (see the pinouts).
