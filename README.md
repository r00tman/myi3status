# myi3status

`myi3status` is a thin wrapper around `i3status` that adds `mpd` status block to the bar.

## Getting Started

```shell
$ git clone https://github.com/r00tman/myi3status
$ cd myi3status && cargo install --path . --force
```

Now you need to add it to your i3/sway configuration file. For example:

```
bar {
    position top
    colors {
        statusline #ffffff
        background #323232
        inactive_workspace #323232 #323232 #5c5c5c
    }
    status_command ~/.cargo/bin/myi3status
    separator_symbol " | "
}
```

Then, you need to set `output-format` in your i3status config depending on what bar you use (e.g., i3bar, lemonbar, etc). For example:

```
general {
        output_format = "i3bar"
        colors = true
        interval = 10
        color_good = "#91bc30"
        color_bad = "#f85858"
}
```

Finally, reload i3: `i3-msg reload`
