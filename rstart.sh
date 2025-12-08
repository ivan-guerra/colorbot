#!/bin/bash

MOUSE_SPEED=50

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
    /var/lib/flatpak/exports/bin/com.adamcake.Bolt &> /dev/null &
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
    xdotool key Shift+Super+Left
    delay
}

function login() {
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/login.json
}

function logout() {
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/logout.json
}

function adjust_camera() {
    # Get maximum overhead view.
    xdotool keydown Up
    sleep 2
    xdotool keyup Up

    # No need to do this the default of the restarted client is sufficient.
    # Scroll up to zoom in.
    # Need to make this work such that the zoom is set to 400 in the game
    # settings.
    # xdotool click --repeat 64 4
    delay

    # Adjust camera slightly to the right.
    # xdotool keydown Right
    # sleep 0.1
    # xdotool keyup Right
    # delay

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
