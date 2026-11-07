# Where should we write the GIF?
Output demo.gif

# Set up a 1200x800 terminal with 18px font.
Set FontSize 18
Set Width 1200
Set Height 800
Set Padding 15
Set LoopOffset 5

Hide
Type `echo "0.5" > sensor && echo "false" > motion && clear`
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
Type "# Integrate with Home Assistant"
Enter
Type "api:"
Enter
Enter
Type "shell:"
Enter
Enter
Type "button:"
Enter
Type " - platform: shell"
Enter
Type `   id: say_hello`
Enter
Type `   name: "Say hello"`
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
Enter
Enter
Type "binary_sensor:"
Enter
Type " - platform: shell"
Enter
Type `   name: "Motion"`
Enter
Type `   command: "cat motion"`
Enter
Type `   update_interval: 1s`
Enter
Type "   on_press:"
Enter
Type "     then:"
Enter
Type "       - button.press: say_hello"


Set TypingSpeed 0.1

# Leave time to read the config.
Sleep 6s

Ctrl+X
Type "y"
Enter


Hide
# Simulate a short motion event while `ubihome run` is recording.
Type `(sh -c "sleep 5; echo true > motion; sleep 1; echo false > motion" > /dev/null 2>&1 &) && clear`
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