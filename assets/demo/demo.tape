Output assets/demo/demo.gif
Output assets/demo/demo.mp4

Set Shell zsh
Set FontSize 22
Set Width 1200
Set Height 600
Set Padding 20
Set Framerate 30
Set Theme "Catppuccin Mocha"

Type "bs sources list"
Enter
Sleep 2s

Type "bs logs tail --source 2280408 --since 1m --query 'message_json.service = auth-service'"
Enter
Sleep 15s
Ctrl+C
Sleep 2s
