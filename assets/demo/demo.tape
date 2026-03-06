Output assets/demo.gif
Output assets/demo.mp4

Set Shell zsh
Set FontSize 16
Set Width 1400
Set Height 700
Set Padding 24
Set Framerate 30
Set Theme "Catppuccin Mocha"

Type "bs logs tail --source 2280408 --since 1m"
Enter
Sleep 8s
Ctrl+C
Sleep 2s

Type "bs logs tail --source 2280408 --since 1m --query 'message_json.service = auth-service'"
Enter
Sleep 8s
Ctrl+C
Sleep 2s
