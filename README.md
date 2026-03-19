# colorbot

A Old School Runescape colorbot utility.

## Requirements

`colorbot` only supports Linux. To build and run this utility, your system must
meet the following requirements:

- rustc >= 1.82.0
- [xdotool][1]

> **Note**: This utility is meant to be used in conjunction with the RuneLite
> plugins Inventory Tags, NPC Indicators, Object Markers, and Menu Entry
> Swapper. Checkout this [blog post][2] for more information.

## Program Usage

`colorbot` is a command line utility. Below is the program usage:

```bash
Usage: colorbot [OPTIONS] <SCRIPT>

Arguments:
  <SCRIPT>  path to bot script

Options:
  -r, --runtime <RUNTIME>  script runtime in seconds [default: 3600]
  -g, --debug              enable logging
  -h, --help               Print help
  -V, --version            Print version
```

`colorbot` has one required argument which is the path to a JSON file containing
mouse events. The events fall under three categories:

- **Color Events**: These events trigger the bot to search for an NPC or object
  with the specified RGB color. If found, the bot will click at the center of
  the object with a randomized coordinate offset. Additionally, a random delay
  is inserted in the range specified by `delay_rng` after the click is
  performed.
- **Keypress Events**: These events trigger the bot to press a specified key on
  the keyboard. You specify keys using [X11 keycodes][3] much like when using
  the `xdotool`. You can also specify a `count` for how many times to press that
  key. A random delay is inserted in the range specified by `delay_rng` after
  the keypress is performed.
- **Mouse Events**: These events trigger the bot to click at a specified
  coordinate. A random offset +/- 5 pixels is automatically applied to the X and
  Y coordinates. A random delay is inserted in the range specified by
  `delay_rng` after the click is performed.
- **Special Events**: These events trigger the bot to perform a special action.
  For example, the `drop_inventory` special event will drop all items in the
  player's inventory (assuming the left click option on inventory items is
  "Drop"). You can add special events to the
  [`special_actions.rs`](src/special_actions.rs) module and then edit the
  [`event.rs`][4] module to add a trigger for that event type.

Below is an example of each event type:

```json
[
  {
    "type": "color",
    "id": "event 1",
    "rgb": [255, 0, 0],
    "delay_rng": [2000, 2250]
  },
  {
    "type": "mouse",
    "id": "event 2",
    "pos": [100, 200],
    "delay_rng": [1500, 1750]
  },
  {
    "type": "keypress",
    "id": "event 3",
    "keycode": "Escape",
    "delay_rng": [750, 1000],
    "count": 1
  },
  {
    "type": "special",
    "id": "drop_inventory"
  }
]
```

Checkout the [scripts/](scripts/) directory for example scripts.

[1]: https://www.semicomplete.com/projects/xdotool/
[2]: https://programmador.com/posts/2025/colorbot/
[3]: https://www.cl.cam.ac.uk/~mgk25/ucs/keysymdef.h
[4]: https://github.com/ivan-guerra/colorbot/blob/master/src/event.rs#L85
