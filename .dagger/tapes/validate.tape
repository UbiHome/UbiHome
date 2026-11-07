Output validate.gif

Set FontSize 18
Set Width 1200
Set Height 900
Set Padding 15
Set TypingSpeed 0.05

Hide
Type `printf 'ubihome:\n  name: demo\n\nshell:\n\nsensor:\n  - platform: shel\n    name: "RAM Usage"\n    command: "free | grep Mem"\n    update_interval: 30s\n' > config.yaml && clear`
Enter
Show

Type "cat config.yaml"
Enter
Sleep 2s

Type "ubihome validate"
Enter
# Leave time to read the error.
Sleep 5s

Type "sed -i 's/platform: shel$/platform: shell/' config.yaml"
Enter
Sleep 500ms

Type "ubihome validate"
Enter
Sleep 4s
