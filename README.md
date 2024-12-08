# colorbot

A Old School Runescape colorbot utility.

### Requirements

`colorbot` only supports Linux. To build and run this utility, your system must
meet the following requirements:

- rustc >= 1.82.0
- [xdotool][1]

### Program Usage

`colorbot` is a command line utility. Below is the program usage:

```bash
A OSRS color bot

Usage: colorbot [OPTIONS] <SCRIPT>

Arguments:
  <SCRIPT>  path to bot script

Options:
  -r, --runtime <RUNTIME>
          script runtime in seconds [default: 3600]
  -d, --mouse-deviation <MOUSE_DEVIATION>
          determines the deviation of the mouse during pathing [default: 30]
  -s, --mouse-speed <MOUSE_SPEED>
          defines the speed of the mouse, lower means faster [default: 3]
  -g, --debug
          enable logging
  -h, --help
          Print help
  -V, --version
          Print version
```

`colorbot` has one required argument which is the path to a JSON file containing
mouse events. The event script must have the following format:

```json
{
  "events": [
    {
      "id": "event1",
      "color": [1, 2, 3],
      "delay_rng": [10, 20]
    }
  ]
}
```

The event script contains a top-level `events` array with one or more mouse
events. Each mouse event has three fields:

- `id`: A string describing the event.
- `color`: A three element array containing the RGB color of the pixel to click
  on.
- `delay_rng`: A two element array containing the minimum and maximum delay in
  milliseconds the script will insert after the click is performed.

Checkout the [scripts/](scripts/) directory for example scripts. Note, this
utility is meant to be used in conjunction with the RuneLite plugins Inventory
Tags and NPC Indicators.

[1]: https://www.semicomplete.com/projects/xdotool/
