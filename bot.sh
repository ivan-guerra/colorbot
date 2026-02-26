#!/etc/profiles/per-user/ieg/bin/bash

COLORBOT_EXE="/home/ieg/dev/colorbot/target/release/colorbot"

function login() {
    $COLORBOT_EXE -r 1 /home/ieg/dev/colorbot/scripts/login.json
}

function logout() {
    $COLORBOT_EXE -r 1 /home/ieg/dev/colorbot/scripts/logout.json
}

function run_bot() {
    local bot_script="/home/ieg/dev/colorbot/scripts/firemaking.json"

    login
    echo "running bot script: $(basename $bot_script) for 3 hours..."
    $COLORBOT_EXE -r 10800 $bot_script
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
        # Don't start another iteration if it's past 11:59 PM
        current_time=$(date +%H:%M)
        if [[ "$current_time" < "23:59" ]]; then
            echo "current time is $current_time, stopping bot session."
            break
        fi

        echo "iteration $i: running bot..."
        run_bot
        random_wait
    done

    echo "bot session completed."
}

main
