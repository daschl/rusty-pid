# Daschl's Rancilio Silvia PID Mod

This repository contains my personal, non commercial, [PID](https://en.wikipedia.org/wiki/PID_controller) modification for the [Rancilio Silvia](https://www.ranciliogroup.com/rancilio/silvia/silvia/) espresso machine.

<img src="/docs/silvia-front.jpg" alt="Machine Front" height="150" />

## Project Overview

The Silvia is a great espresso machine, but the built-in temperature sensor causes an oscillation of over 10 degrees celsius:

<img src="/docs/oscillation.png" alt="Regular Oscillation" height="150" />

Over time, users have come up with the idea of [temperature surfing](https://www.youtube.com/watch?v=IYMF9yY-TR0), but that's still error prone and not consistent. A PID controller allows to keep the temperature stable of +- 1 degrees celsius and also recover quickly after pulling a shot.