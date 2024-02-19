# wavrw
`wavrw` is a command line tool to read (and someday write) WAV files with a focus on visualizing the structure of files and parsing metadata.

It's initially intended as a personal learning project with aspirations to be generally useful. It's currently in *very* early days and unlikely to be useful to anyone else. (It doesn't yet even parse the basics of WAV files.)

`wavrw` answers questions about WAV files: 
  * What is *actually* in this file?
  * Does another program preserve all WAV chunks when saving and exporting files? 

## Help overview

```
WAV file metadata read/write utility

Usage: wavrw <COMMAND>

Commands:
  view    Summarize WAV file structure and metadata
  chunks  List chunks contained in files, one per line
  help    Print this message or the help of the given subcommand(s)

Global Options:
  -h, --help     Print help
  -V, --version  Print version

```
