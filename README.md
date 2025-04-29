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
      "type": "keypress",
      "action": "F1"
      "id": "event1",
      "delay_rng": [10, 20]
    },
    {
      "type": "mouse",
      "action": "left_click"
      "id": "event2",
      "color": [1, 2, 3],
      "delay_rng": [10, 20]
    },
    {
      "type": "mouse",
      "action": "left_click"
      "id": "event2",
      "color": [1, 2, 3],
      "delay_rng": [10000, 20000]
      "skip_if_vanished": true
    },
    {
      "type": "mouse",
      "action": "shift_click"
      "id": "event1",
      "color": [1, 2, 3],
      "delay_rng": [10, 20],
      "count": 27
    }
  ]
}
```

The event script contains a top-level `events` array with one or more mouse
events.

- `type`: The type of event to execute, you choose between `keypress` or `mouse`
- `action`: The type of click or keypress you want. 
  - Mouse supports `left_click`,`right_click` and `shift_click` (shift left click) 
  - Keypress supports `F1` to `F12`, `Escape`, and key combinations `Ctrl+Shift+Right` (does not need the color parameter)
- `id`: A string describing the event.
- `color`: A three element array containing the RGB color of the pixel to click
  on. For `mouse` type only.
- `delay_rng`: A two element array containing the minimum and maximum delay in
  milliseconds the script will insert after the click is performed.
- `count`: The number of repetitions you want, by default is `1` and can be ommited in the json.
- `skip_if_vanished`: Only for `mouse` type, can be ommited, by default is set to `false`. If the target color is not present in the screen anymore, it will skip to the next event.

Checkout the [scripts/](scripts/) directory for example scripts.<br>
Note, this utility is meant to be used in conjunction with the RuneLite plugins like:<br> 
- Inventory Tags
- Bank Highlighter
- NPC Indicators
- Object Markers
- Menu Entry Swapper
- Canvas or Brush Markers can also be helpful

I recommend in RS to `Settings -> Controls -> Enable "Shift click to drop items" and "Esc closes the interface"` <br> 
And `Settings -> Warnings -> Confirmations -> Disable "World Switcher confirmation"` (to use along the World Hopper Plugin)

Checkout this
[blog post][2] for more information.

[1]: https://www.semicomplete.com/projects/xdotool/
[2]: https://programmador.com/posts/2025/colorbot/
