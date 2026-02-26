#!/etc/profiles/per-user/ieg/bin/bash

MOUSE_SPEED=50
COLORBOT_EXE="/home/ieg/dev/colorbot/target/release/colorbot"

function delay() {
    sleep 0.5
}

function kill_client() {
    # Click on the left hand side of a 1920x1080 screen
    xdotool mousemove 100 540 click 1
    delay

    # Close the client
    xdotool key Super+c
    sleep 2
}

function load_client() {
    # Start the game launcher
    bolt-launcher &> /dev/null &
    sleep 15

    # Click on the right hand side of a 1920x1080 screen on launcher
    xdotool mousemove 1800 540 click 1
    delay

    # Press the play button
    xdotool mousemove 1441 230 click 1
    sleep 30

    # Click in the center of 1920x1080 screen on launcher
    xdotool mousemove 960 540 click 1
    delay

    # Close the launcher
    xdotool key Super+c
    delay

    # Click on the right hand side of a 1920x1080 screen to focus client
    xdotool mousemove 1800 540 click 1
    delay

    # Move client to the left side of the screen
    xdotool key Shift+Super+h
    delay
}

function login() {
    $COLORBOT_EXE -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/login.json
}

function logout() {
    $COLORBOT_EXE -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/logout.json
}

function adjust_camera() {
    # Get maximum overhead view.
    xdotool keydown Up
    sleep 2
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
