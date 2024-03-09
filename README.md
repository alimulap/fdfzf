I use this for bash function like this:

```bash 
function sdir() {
    cd $(fdfzf -t d "$@")
}

function sfile() {
    xdg-open $(fdfzf -t f "$@")
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
