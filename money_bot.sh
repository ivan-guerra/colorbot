#!/bin/bash

DISPLAY=:1
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
    colorbot -s $MOUSE_SPEED -r 1 /home/ieg/dev/colorbot/scripts/logout2.json
}

function run_sailing_bot() {
    login
    colorbot -s $MOUSE_SPEED -r 10800 /home/ieg/dev/colorbot/scripts/salvage.json
    logout
}

# Session 1   07:00 AM   10:00 AM   Run
# Break 1     10:00 AM   11:30 AM   Break
# Session 2   11:30 AM   02:30 PM   Run
# Break 2     02:30 PM   04:00 PM   Break
# Session 3   04:00 PM   07:00 PM   Run
# Break 3     07:00 PM   08:30 PM   Break
# Session 4   08:30 PM   11:30 PM   Run
function main {
    log_info "starting sailing bot"

    local i=1
    while true; do
        current_hour=$(date +%H)

        if [ "$current_hour" -ge 23 ] || [ "$current_hour" -lt 7 ]; then
            log_info "it's between 11 pm and 7 am, stopping the bot."
            break
        fi

        log_info "running sailing bot iteration ${i}"
        run_sailing_bot
        log_info "completed sailing bot iteration ${i}, sleeping for 1.5 hours"
        sleep 5400

        i=$((i + 1))
    done

    log_info "sailing bot completed."
}

main
