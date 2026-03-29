phi/xi is a terminal-based modal text editor.

as of now, supports insert mode and command mode.

press `Esc` to enter command mode, `Esc` again to be back in insert mode.

currently available commands:
- `w` (write)
    - `w <filename>` (save as)
- `q` (quit)
- `b` (new buffer)
    - `b <buf_number>` (open a specified buffer)
- `e <filename>` (opens the specified file)
- `undo`
- `redo`
- `v` (toggle selection)
- `y` (yank/copy selection)
- `p` (paste)

navigation is via arrow keys only, for now.
