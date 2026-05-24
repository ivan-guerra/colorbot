# ColorBot

A Old School RuneScape bot that uses color detection and image recognition to
automate in-game actions.

## Features

- **Color Detection**: Work in conjunction with RuneLite Object Markers or NPC
  Indicators plugins to click within the boundaries of colored outlines.
- **Image Recognition**: Find and interact with in game objects using parallel
  template matching. You can make your own templates with any screen capture
  tool.
- **Human-like Mouse Movement**: WindMouse algorithm for natural cursor paths.
- **Keyboard Automation**: Simulate any key input supported by X11.
- **JSON-based Scripting**: Define complex automation sequences with
  customizable delays and repetitions.

## Prerequisites

- Linux operating system
- X11 display server
- `xdotool` - for keyboard and mouse control
- A screen capture tool (e.g., `scrot`) for creating image templates of your own

## Usage

```bash
colorbot path/to/script.json
```

Run `colorbot --help` for more options.

## Script Format

Scripts are defined in JSON format with an array of events. Each event has
common properties and type-specific parameters.

### Common Event Properties

All events support these properties:

- `type`: The event type (required) - one of: `keypress`, `color`, or `image`
- `id`: A descriptive identifier for logging purposes (required)
- `count`: Number of times to execute this event (optional, defaults to 1)
- `delay_rng`: Random delay range in milliseconds `[min, max]` after execution

### Event Types

#### KeyPress Event

Simulates keyboard input using xdotool key names.

```json
{
  "type": "keypress",
  "id": "press_escape",
  "count": 1,
  "keycode": "Escape",
  "delay_rng": [750, 1000]
}
```

`keycode` repesents the key to press (xdotool format, e.g., "a", "Escape",
"Return", "ctrl+c").

#### Color Detection Event

Finds and clicks on a specific RGB color on screen.

```json
{
  "type": "color",
  "id": "click_blue_button",
  "count": 1,
  "rgb": [52, 152, 219],
  "delay_rng": [500, 750]
}
```

`rgb` represents the target RGB color values `[r, g, b]` (0-255). This is best
used with the outline function of the RuneLite Object Markers or NPC Indicators
plugins. The bot is smart enough to click within the boundaries of the colored
outline with randomized offsets to mimic human behavior.

#### Image Recognition Event

Locates and clicks on a UI element using template matching.

```json
{
  "type": "image",
  "id": "click_submit_button",
  "count": 1,
  "image_path": "templates/submit_button.png",
  "delay_rng": [500, 1000]
}
```

`image_path` is the path to the template image file (PNG format recommended).
The bot captures the screen and searches for the template image. If found, it
clicks within the matched area with randomized offsets.

### Example Script

```json
[
  {
    "type": "keypress",
    "id": "open_menu",
    "keycode": "Escape",
    "delay_rng": [750, 1000]
  },
  {
    "type": "color",
    "id": "click_settings",
    "rgb": [52, 152, 219],
    "delay_rng": [500, 750]
  },
  {
    "type": "image",
    "id": "click_confirm",
    "image_path": "templates/confirm_button.png",
    "delay_rng": [1000, 1500]
  },
  {
    "type": "keypress",
    "id": "repeat_action",
    "count": 5,
    "keycode": "space",
    "delay_rng": [200, 300]
  }
]
```
