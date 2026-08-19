## Developer tools

<img src=".github/app_screen.png" alt="Dev tools screen" />

1) Formatting XML, JSON
2) Escape/Unescape XML, JSON, URL
3) Compare text
4) Share file
5) Supporting large files
6) Rest client
Web version: https://dev-tools-rust.vercel.app

There are also standalone applications for Windows, Linux, and Mac OS.

<img src=".github/app_rest_client.png" alt="Dev tools rest client screen" />

https://github.com/DimetriusJonson/dev-tools/releases

The standalone app listens on port 3005 by default. This can be changed using command-line arguments.<br>
For example,<br>
    webdev_useful_tools.exe --port 3067<br>
<br>
The standalone app also uses the default remote server "https://dev-tools-rust.vercel.app" for the "Share File" feature.
The server address can be changed using a command-line argument.<br>
For example,<br>
    webdev_useful_tools.exe --remote-server-url https://custom-server<br>
<br>
By default, the standalone application starts a local server.<br>
You can disable server startup with the --no-start-server option.<br>
For example, <br>
    webdev_useful_tools.exe --no-start-server<br>
The application will work with the server specified in the --remote-server-url option.

Useful commands:

1) Build standolone app

just build-windows<br>
just build-linux<br>
just build-macos<br>
