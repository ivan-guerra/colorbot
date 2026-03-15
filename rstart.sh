#!/etc/profiles/per-user/ieg/bin/bash

COLORBOT_EXE="/home/ieg/dev/colorbot/target/release/colorbot"

function delay() {
    if [ -n "$1" ]; then
        sleep "$1"
    else
        sleep 0.5
    fi
}

function press_key() {
    local key="$1"

    xdotool key "$key"
    delay
}

function left_click() {
    local x="$1"
    local y="$2"

    xdotool mousemove "$x" "$y" click 1
    delay
}

function kill_client() {
    # Click on the left hand side of a 1920x1080 screen
    left_click 100 540

    # Close the client
    press_key Super+c
    delay 2
}

function load_client() {
    # Start the game launcher
    bolt-launcher &> /dev/null &
    delay 10

    # Click on the right hand side of a 1920x1080 screen on launcher
    left_click 1800 540

    # Navigate to the player select drop down list using tab key
    for _ in {1..9}; do
        press_key Tab
    done

    # Expand the player select drop down list
    press_key space

    # Select the player based on the display number
    if [ "$DISPLAY" = ":1" ]; then
        # Select Antikles
        for _ in {1..3}; do
            press_key Up
        done
        # Select Antikles
        press_key Return
    elif [ "$DISPLAY" = ":2" ]; then
        for _ in {1..3}; do
            press_key Down
        done
        # Select Aldur07
        press_key Return
    fi

    # Press the play button
    left_click 1441 230
    delay 15

    # Click in the center of 1920x1080 screen on launcher
    left_click 960 540

    # Close the launcher
    press_key Super+c

    # Click on the right hand side of a 1920x1080 screen to focus client
    left_click 1800 540

    # Move client to the left side of the screen
    press_key Shift+Super+h
}

function login() {
    $COLORBOT_EXE -r 1 /home/ieg/dev/colorbot/scripts/login.json
}

function logout() {
    $COLORBOT_EXE -r 1 /home/ieg/dev/colorbot/scripts/logout.json
}

function adjust_camera() {
    # Get maximum overhead view.
    xdotool keydown Up
    delay 2
    xdotool keyup Up

    # Scroll down to zoom out.
    xdotool click --repeat 64 5
    delay

    # Avoid bug where seemingly shift is pressed.
    xdotool key Shift
}

function main() {
    echo "killing client"
    kill_client

    echo "loading client"
    load_client

    echo "logging in"
    login

    echo "adjusting camera"
    adjust_camera

    echo "logging out"
    logout

    echo "done"
}

main
