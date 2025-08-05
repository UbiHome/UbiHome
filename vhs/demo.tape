# Where should we write the GIF?
Output demo.gif

# Set up a 1200x600 terminal with 46px font.
Set FontSize 18
Set Width 1200
Set Height 700
Set Padding 15
Set LoopOffset 5

Hide
Type `echo "0.5" > sensor`
Enter
Type "rm -f config.yaml && clear"
Enter
Show


Type "nano config.yaml"
Enter
Set TypingSpeed 0
Type "ubihome:"
Enter
Type "  name: 'demo'"
Enter
Enter
Type "logger:"
Enter
Type "  level: info"
Enter
Type "  logs:"
Enter
Type "    web_server: debug"
Enter
Enter
Type "# Integrate with Home Assistant"
Enter
Type "api:"
Enter
Enter
Type "# Allow simple HTTP Triggers"
Enter
Type "web_server:"
Enter
Enter
Type "shell:"
Enter
Enter
Type "button:"
Enter
Type " - platform: shell"
Enter
Type `   name: "Example button"`
Enter
Type `   command: "echo \"Hello World\" > hello"`
Enter
Enter
Type "sensor:"
Enter
Type " - platform: shell"
Enter
Type `   name: "Example sensor"`
Enter
Type `   command: "cat sensor"`
Enter
Type `   update_interval: 1s`


Set TypingSpeed 0.1

Sleep 1s

Ctrl+X
Type "y"
Enter


Hide
Type "nohup sh ./trigger_button.sh > /dev/null 2> /dev/null < /dev/null && clear"
Enter
Show

# Env RUST_LOG "debug"
# Type a command in the terminal.
Type "ubihome run"

# Pause for dramatic effect...
Sleep 500ms

# Run the command by pressing enter.
Enter

# Admire the output for a bit.
Sleep 10s

Ctrl+C

Type "cat hello"
Enter
Sleep 2s