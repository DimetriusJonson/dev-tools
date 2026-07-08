## Developer tools

<img src=".github/app_screen.png" alt="Tauri" />

1) Formatting XML, JSON
2) Escape/Unescape XML, JSON, URL
3) Supporting large files

Web version: https://leptos-devtools.up.railway.app/

There are also standalone applications for Windows, Linux, and Mac OS.

https://github.com/DimetriusJonson/dev-tools/releases

The standalone app listens on port 3005 by default. This can be changed using command-line arguments.
For example,
    dev_tools.exe --port=3067

The standalone app also uses the default remote server "https://leptos-devtools.up.railway.app" for the "Share File" feature.
The server address can be changed using a command-line argument.
For example,
    dev_tools.exe --remote-server-url=https://custom-server