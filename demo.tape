# vhs demo tape - drives the real TUI against live devices.
# Only interacts with "Floor Lamp" (device #5 in the list).
Output demo.gif

Set FontSize 15
Set Width 1200
Set Height 760
Set TypingSpeed 120ms
Set Padding 12

Type "./target/release/govee-tui"
Enter
# initial device load
Sleep 5s

# navigate to Floor Lamp (5th in list)
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 1.5s

# power from the list: first press loads state (honest UX), second toggles
Space
Sleep 3.5s
Space
Sleep 3s
Space
Sleep 3s

# into the detail pane
Enter
Sleep 3s

# brightness up (debounced - two presses, one API call)
Type "k"
Sleep 250ms
Type "k"
Sleep 3s
# and back down
Type "j"
Sleep 250ms
Type "j"
Sleep 3s

# color picker: RGB mode, then browse named colors (cancel, no apply)
Type "c"
Sleep 2s
Tab
Sleep 1.5s
Down
Sleep 500ms
Down
Sleep 500ms
Right
Sleep 1.5s
Escape
Sleep 1s

# scene picker: load, browse, apply
Type "s"
Sleep 4s
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 1s
Enter
Sleep 3.5s

# help modal
Type "?"
Sleep 3.5s
Escape
Sleep 1s

Type "q"
Sleep 1s
