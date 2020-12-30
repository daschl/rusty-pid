# Daschl's Rancilio Silvia PID Mod

<img src="/docs/silvia-front.jpg" alt="Machine Front" height="250" />

This repository contains my personal, non commercial, [PID](https://en.wikipedia.org/wiki/PID_controller) modification for the [Rancilio Silvia](https://www.ranciliogroup.com/rancilio/silvia/silvia/) espresso machine.

**Feel free to use any code in this repository for your project as well, but I do not provide any guarantees or support whatsoever. If you make modifications to your coffee machine, you will void your warranty! Also, working with 230V can be dangerous and should only be done if you know what you are doing!**

## Project Overview

The Silvia is a great espresso machine, but the built-in temperature sensor causes an oscillation of over 10 degrees celsius:

<img src="/docs/oscillation.png" alt="Regular Oscillation" height="200" />

Over time, users have come up with the idea of [temperature surfing](https://www.youtube.com/watch?v=IYMF9yY-TR0), but that's still error prone and not very consistent. A PID controller allows to keep the temperature stable of +- 1 degrees celsius and also recover quickly after pulling a shot.

You can buy off the shelf kits from companies like Auber, but I wanted to implement it myself as a learning experience. The main inspiration and ideas came from the [Rancilio-PID](http://rancilio-pid.de/) project, for which I thank them a lot. They open-sourced the complete arduino implementation on Github and made it possible to adapt it for the Rust ecosystem.

This repository right now contains two modules:

 - `controller`: the main embedded controller which lives inside the machine and is the heart and brain.
 - `ui`: working on a iOS app to monitor and configure the controller via BLE (Bluetooth Low Energy).

## Controller

The controller is a [Adafruit Feather nRF52840 Express](https://www.adafruit.com/product/4062) that runs [RTIC](https://rtic.rs). It measures the boiler temperature through a [TSIC 306](https://www.ist-ag.com/sites/default/files/DTTSic20x_30x_E.pdf) sensor and controls the heater through a [Carlo Gavazzi RA4850 SSR](https://eu.mouser.com/ProductDetail/Carlo-Gavazzi/RA4850-D12?qs=xZ%2FP%252Ba9zWqbxbkVxBjta9Q==).

Both controller and SSR are located in the front section of the machine, behind the front plate:

<img src="/docs/controller-and-ssr.jpg" alt="Controller and SSRs" height="400" />

## I want to do the same!

If you are interested, right now the best way to get in touch is probably via [Twitter](https://twitter.com/daschl), because I haven't had the chance to write up a full guide. Also, check out my [blog](https://nitschinger.at/) where I try to write up my experiences and progress for this project.