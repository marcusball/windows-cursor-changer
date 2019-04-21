# Windows Cursor Changer

## Dev Usage 

Build: 

```
cargo build
```

Run:

```
cargo run
```

## Release builds

```
cargo build --release
```

```
cargo run --release
```


## Configuration

This application expects a `cursor.toml` file to be presents in the directory in which the executable is located. 
The format of the toml file is as such: 

```
[[cursor]]
name = "dog"
path = "sissi.ani"

[[cursor]]
name = "big"
path = "big.cur"

[[application]]
cursor = "dog"
path = "powershell.exe"
```

For each cursor graphic, there should be a `[[cursor]]` table, giving it a unique `name`, and then the path to the .ani/.cur file. 

For each application, add an `[[application]]` table, and specify the `cursor` (by `name`) that should be used when over that application's windows.
When checking if the cursor is over the desired application, this app will check whether an executable's full path *ends with* the `path` specified
in the config file. So, you may use `path = "my-app.exe"`, or `path = "subfolder\my-app.exe"`, or even the full absolute path. 