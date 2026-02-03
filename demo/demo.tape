# VHS tape for JSONQuill demo
# This demonstrates the major features of jsonquill

Output assets/demo.mp4

Set Theme {
    "name": "default-dark",
    "black": "#151515",
    "red": "#ac4141",
    "green": "#7e9968",
    "yellow": "#e5b567",
    "blue": "#6b99bc",
    "magenta": "#9c637d",
    "cyan": "#7dd5cf",
    "white": "#d0d0d0",
    "brightBlack": "#505050",
    "brightRed": "#ac4141",
    "brightGreen": "#7e9968",
    "brightYellow": "#e5b567",
    "brightBlue": "#6b99bc",
    "brightMagenta": "#9c637d",
    "brightCyan": "#7dd5cf",
    "brightWhite": "#f5f5f5",
    "background": "#292929",
    "foreground": "#efebeb",
    "selection": "#303030",
    "cursor": "#6b99bc"
  }

Set FontSize 12
Set Width 1024
Set Height 768
Set TypingSpeed 50ms
Set PlaybackSpeed 0.8

# Start jsonquill
Type "jsonquill demo/demo-data.json"
Enter
Sleep 2s

# Basic navigation with j/k
Type ":"
Sleep 200ms
Type "Navigate with j/k (down/up)"
Sleep 1.5s
Escape
Sleep 300ms
Type "jjjjjj"
Sleep 1s
Type "kkk"
Sleep 1s

# Collapse with h
Type ":"
Sleep 200ms
Type "Press 'h' to collapse node"
Sleep 1.5s
Escape
Sleep 300ms
Type "h"
Sleep 1s

# Expand with l
Type ":"
Sleep 200ms
Type "Press 'l' to expand node"
Sleep 1.5s
Escape
Sleep 300ms
Type "l"
Sleep 1s

# Navigate into expanded node
Type "jj"
Sleep 800ms

# Jump commands
Type ":"
Sleep 200ms
Type "Jump to top with 'gg'"
Sleep 1.5s
Escape
Sleep 300ms
Type "gg"
Sleep 1s

Type ":"
Sleep 200ms
Type "Jump to bottom with 'G'"
Sleep 1.5s
Escape
Sleep 300ms
Type "G"
Sleep 1s

Type ":"
Sleep 200ms
Type "Jump to line 5 with '5G'"
Sleep 1.5s
Escape
Sleep 300ms
Type "5G"
Sleep 1s

# Sibling navigation
Type ":"
Sleep 200ms
Type "Navigate siblings with } and {"
Sleep 1.5s
Escape
Sleep 300ms
Type "}}"
Sleep 800ms
Type "{"
Sleep 1s

Type ":"
Sleep 200ms
Type "Fully collapse with 'C'"
Sleep 1.5s
Escape
Sleep 300ms
Type "1G"
Sleep 1s
Type "C"
Sleep 1.5s

# Full expand/collapse
Type ":"
Sleep 200ms
Type "Fully expand with 'E'"
Sleep 1.5s
Escape
Sleep 300ms
Type "E"
Sleep 1.5s

# Search demonstration
Type ":"
Sleep 200ms
Type "Search with / then type pattern"
Sleep 1.5s
Escape
Sleep 300ms
Type "/"
Sleep 500ms
Type "price"
Sleep 500ms
Enter
Sleep 1.5s

# Edit a value
Type "gg"
Sleep 300ms
Type "7j"
Sleep 500ms
Type ":"
Sleep 200ms
Type "Edit value with 'e'"
Sleep 1.5s
Escape
Sleep 300ms
Type "e"
Sleep 500ms
Backspace 6
Type "1299.99"
Sleep 500ms
Enter
Sleep 1.5s

# Add a new field
Type "j"
Sleep 300ms
Type ":"
Sleep 200ms
Type "Add field with 'i' (prompts for key and value)"
Sleep 1.5s
Escape
Sleep 300ms
Type "i"
Sleep 500ms
Type "warranty"
Sleep 300ms
Enter
Sleep 300ms
Type "2 years"
Sleep 300ms
Enter
Sleep 1.5s

# Yank and paste
Type "gg"
Sleep 300ms
Type "jjjjjjjjjj"
Sleep 500ms
Type ":"
Sleep 200ms
Type "Yank (copy) with 'yy'"
Sleep 1.5s
Escape
Sleep 300ms
Type "yy"
Sleep 1s
Type "jjjjj"
Sleep 300ms
Type ":"
Sleep 200ms
Type "Paste with 'p'"
Sleep 1.5s
Escape
Sleep 300ms
Type "p"
Sleep 1.5s

# Delete and undo
Type ":"
Sleep 200ms
Type "Delete with 'dd'"
Sleep 1.5s
Escape
Sleep 300ms
Type "dd"
Sleep 1.5s

Type ":"
Sleep 200ms
Type "Undo with 'u'"
Sleep 1.5s
Escape
Sleep 300ms
Type "u"
Sleep 1.5s

# Visual mode
Type ":"
Sleep 200ms
Type "Visual mode with 'v', then j/k to select, 'y' to yank"
Sleep 2s
Escape
Sleep 300ms
Type "v"
Sleep 500ms
Type "jj"
Sleep 500ms
Type "y"
Sleep 1s
Escape
Sleep 500ms

# JSONPath search
Type ":"
Sleep 200ms
Type "JSONPath search - finding all inventory names"
Sleep 1.5s
Escape
Sleep 300ms
Type ":path $.store.inventory[*].name"
Enter
Sleep 1.5s
Type "n"
Sleep 800ms
Type "n"
Sleep 800ms
Escape
Sleep 500ms

# Theme switching
Type ":"
Sleep 200ms
Type "Switch themes with ':theme <name>', or :theme for picker"
Sleep 1.5s
Escape
Sleep 300ms
Type ":theme"
Enter
Sleep 1s
Type "j"
Sleep 1s
Type "j"
Sleep 1s
Type "j"
Enter
Sleep 1.5s

# Show help
Type ":"
Sleep 200ms
Type "help or <F1> for help overlay"
Sleep 1.5s
Escape
Sleep 300ms
Type ":help"
Enter
Sleep 1s
Type "jjjjj"
Sleep 500ms
Type "jjjjj"
Sleep 500ms
Type "kkk"
Sleep 500ms
Escape
Sleep 1s

# Save and quit
Type ":w demo-output.json"
Enter
Sleep 1.5s

Type ":q"
Enter
Sleep 1s

# Show the saved file
Type "cat demo-output.json | head -20"
Enter
Sleep 3s
