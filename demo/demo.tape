# VHS demo tape for bs CLI
# Prerequisites:
#   1. Start the log generator in another terminal:
#      BS_SOURCE_TOKEN=xxx ./demo/log-generator.sh
#   2. Wait ~30 seconds for logs to accumulate
#   3. Run: vhs demo/demo.tape
#
# Set BS_DEMO_SOURCE to your source ID before running.

Output demo/demo.gif
Output demo/demo.mp4

Set Theme "Catppuccin Mocha"
Set FontSize 15
Set Width 1200
Set Height 600
Set Padding 20
Set TypingSpeed 50ms

# Start with a clean terminal
Hide
Type "clear"
Enter
Show

Sleep 500ms

# Show the tail command
Type "bs logs tail --source ${BS_DEMO_SOURCE:-12345} --since 1m"
Sleep 500ms
Enter

# Let logs stream in
Sleep 15s

# Ctrl+C to stop
Ctrl+C
Sleep 1s

# Now show a query
Type "bs logs query 'level:ERROR' --source ${BS_DEMO_SOURCE:-12345} --since 5m"
Sleep 500ms
Enter
Sleep 3s

# Show JSON output piped to jq
Type "bs logs query 'level:ERROR' --source ${BS_DEMO_SOURCE:-12345} --since 5m -o json | jq '.message'"
Sleep 500ms
Enter
Sleep 3s
