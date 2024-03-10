I use this for bash function like this:

```bash 
function sdir() {
    local SELECTED=$(fdfzf -t d "$@")
    if [ -n "$SELECTED" ]; then
        cd "$SELECTED"
    fi
}

function sfile() {
    local SELECTED=$(fdfzf -t f "$@")
    if [ -n "$SELECTED" ]; then
        kde-open "$SELECTED"
    fi
}
```

and then i do:

```bash
sdir ~/onedrive -d 4
```

or just 

```bash
sdir
```

works

there is also config

```bash
fdfzf -c ./config-example.toml -p custom_name_literally_anything
```

`-c` needs to be a path to a toml file, and `-p` is a profile name define inside the config file. You can se the [config example here](https://github.com/alimulap/fdfzf/blob/main/config-example.toml).

if not supplied, fdfzf will search for `~/.config/fdfzf/config.toml` and use the `default` profile.

also there is `-H` to show hidden files.

```bash
fdfzf -H
```

