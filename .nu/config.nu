$env.config.show_banner = false
$env.config.buffer_editor = "hx"

$env.PROMPT_INDICATOR = ""
$env.PROMPT_INDICATOR_VI_INSERT = ": "
$env.PROMPT_INDICATOR_VI_NORMAL = "〉"
$env.PROMPT_MULTILINE_INDICATOR = "::: "

$env.STARSHIP_SHELL = "nu"

def create_left_prompt [] {
    starship prompt --cmd-duration $env.CMD_DURATION_MS $'--status=($env.LAST_EXIT_CODE)'
}

def create_right_prompt [] {
    starship prompt --right
}

$env.PROMPT_COMMAND = { || create_left_prompt }
$env.PROMPT_COMMAND_RIGHT = { || create_right_prompt }

def show_greeter [] {
    clear --keep-scrollback

    print '|>     _     _           _                __             '
    print '|>    | |   | |         (_)              / _|            '
    print '|>    | |__ | | __ _ _____ _ __   __ _  | |_ __ _ _ __   '
    print '|>    |  _ \| |/ _` |_  / |  _ \ / _` | |  _/ _` |  _ \  '
    print '|>    | |_) | | (_| |/ /| | | | | (_| | | || (_| | | | | '
    print '|>    |_.__/|_|\__,_/___|_|_| |_|\__, | |_| \__,_|_| |_| '
    print '|>                                __/ |                  '
    print '|>                               |___/                   '
    print '|>                                                       '

    print $"|>    (ansi yellow_bold)Tools:(ansi reset)"
    print $"|>    (rustc --version)"
    print $"|>    (cargo --version)"
    print $"|>    (laze --version)"
    print '|>'

    print $"|>    (ansi yellow_bold)Building:(ansi reset)"
    print '|>    laze build -b rpi-pico'
    print '|>'

    print $"|>    (ansi yellow_bold)Running:(ansi reset)"
    print '|>    laze build -b rpi-pico run'
    print '|>'
}

show_greeter
