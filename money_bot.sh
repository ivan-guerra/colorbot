#!/bin/bash

MOUSE_SPEED=50

function log_info() {
    local timestamp=

    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $1"
}

function login() {
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/login.json
}

function logout() {
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/logout.json
}

function run_herb_bot() {
    login
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/farming_setup.json
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/farming.json
    logout
}

function run_runecraft_bot() {
    login
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/runecraft_setup.json
    colorbot -s $MOUSE_SPEED -r 3600 /home/ieg/dev/colorbot/scripts/runecraft.json
    logout
}

function main {
    log_info "starting money bot"

    herb_run_count=0
    while true; do
        current_hour=$(date +%H)

        if [ "$current_hour" -ge 23 ] || [ "$current_hour" -lt 7 ]; then
            log_info "it's between 11 pm and 7 am, stopping the bot."
            break
        fi

        log_info "running herb bot..."
        run_herb_bot
        herb_run_count=$((herb_run_count + 1))
        log_info "herb bot completed (run $herb_run_count)."

        # Run runecraft bot after the 2nd, 4th, and 6th herb runs
        if [ "$herb_run_count" -eq 2 ] || [ "$herb_run_count" -eq 4 ] || [ "$herb_run_count" -eq 6 ]; then
            log_info "running runecraft bot after herb run $herb_run_count..."
            run_runecraft_bot
            log_info "runecraft bot completed."

            log_info "waiting for 30 minutes before next herb run..."
            sleep 1800
        else
            log_info "waiting for 90 minutes before next herb run..."
            sleep 5400
        fi

    done

    log_info "money bot completed."
}

main
