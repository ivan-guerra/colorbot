#!/etc/profiles/per-user/ieg/bin/bash

COLORBOT_EXE="/home/ieg/dev/colorbot/target/release/colorbot"

function login() {
    $COLORBOT_EXE -r 1 /home/ieg/dev/colorbot/scripts/login.json
}

function logout() {
    $COLORBOT_EXE -r 1 /home/ieg/dev/colorbot/scripts/logout.json
}

function run_bot() {
    local fm_setup_script="/home/ieg/dev/colorbot/scripts/firemaking_setup.json"
    local fm_script="/home/ieg/dev/colorbot/scripts/firemaking.json"

    login
    echo "running bot script: firemaking script for 3 hours..."
    $COLORBOT_EXE -r 1 $fm_setup_script
    $COLORBOT_EXE -r 10800 $fm_script
    logout
}

function random_wait() {
    local min=5400  # 1.5 hours in seconds
    local max=7200  # 2 hours in seconds
    local sleep_time=$((RANDOM % (max - min + 1) + min))

    echo "waiting for $((sleep_time / 60)) minutes before next run..."
    sleep $sleep_time
}

function main {
    echo "starting bot session"

    for i in {1..4}; do
        # Check if current time is between 11 pm and 7 am
        current_hour=$(date +%H)
        if [ "$current_hour" -ge 23 ] || [ "$current_hour" -lt 7 ]; then
            log_info "it's between 11 pm and 7 am, stopping the bot."
            break
        fi

        echo "iteration $i: running bot..."
        run_bot
        random_wait
    done

    echo "bot session completed."
}

main
