# Daschl's Rancilio Silvia PID Mod

This repository contains my personal, non commercial, [PID](https://en.wikipedia.org/wiki/PID_controller) modification for the [Rancilio Silvia](https://www.ranciliogroup.com/rancilio/silvia/silvia/) espresso machine.

**Feel free to use any code in this repository for your project as well, but I do not provide any guarantees or support whatsoever. If you make modifications to your coffee machine, you will void your warranty! Also, working with 230V can be dangerous and should only be done if you know what you are doing!**

<img src="/docs/silvia-front.jpg" alt="Machine Front" height="200" />

## Project Overview

The Silvia is a great espresso machine, but the built-in temperature sensor causes an oscillation of over 10 degrees celsius:

<img src="/docs/oscillation.png" alt="Regular Oscillation" height="200" />

Over time, users have come up with the idea of [temperature surfing](https://www.youtube.com/watch?v=IYMF9yY-TR0), but that's still error prone and not very consistent. A PID controller allows to keep the temperature stable of +- 1 degrees celsius and also recover quickly after pulling a shot.

You can buy off the shelf kits from companies like Auber, but I wanted to implement it myself as a learning experience. The main inspiration and ideas came from the [Rancilio-PID](http://rancilio-pid.de/) project, for which I thank them a lot. They open-sourced the complete arduino implementation on Github and made it possible to adapt it for the Rust ecosystem.