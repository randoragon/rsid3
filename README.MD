# rsid3

A simple command-line ID3v2 tag editor designed with scripting in mind.

## Features

- Print all frames of a file,
- Print, set or delete chosen frames.

## Examples

```sh

# Printing frames
rsid3 file.mp3                               # Print all frames
rsid3 --TIT2 file.mp3                        # Print TIT2 (title)
rsid3 --TXXX some_desc file.mp3              # Print TXXX[some_desc]
rsid3 --COMM some_desc eng file.mp3          # Print COMM[some_desc](eng)
rsid3 --COMM some_desc first file.mp3        # Print COMM[some_desc] and any first language found
rsid3 -d, --TIT2 --TPE1 --TALB file.mp3      # Print title, artist and album, delimited by comma

# Setting frames
rsid3 --TIT2= 'My Title' file.mp3              # Set TIT2
rsid3 --TXXX= some_desc 'some value' file.mp3  # Set TXXX[some_desc]
rsid3 --COMM= some_desc eng file.mp3           # Set COMM[some_desc](eng)

# Deleting frames
rsid3 --TIT2- file.mp3                    # Delete TIT2
rsid3 --TXXX- some_desc file.mp3          # Delete TXXX[some_desc]
rsid3 --COMM- some_desc eng file.mp3      # Delete COMM[some_desc](eng)

# All types of options can be mixed; prints always happen first, then sets, then deletes
rsid3 --TIT2 --TIT2= new_title file.mp3  # Print old title and set new title

# Any number of target files can be specified. The following command prints "artist - title",
# removes COMM[some_desc](eng) and sets TXXX[abc] to "def" for two files, all in one command:
rsid3 -d ' - ' --TPE1 --TIT2 --COMM- some_desc eng --TXXX= abc def file1.mp3 file2.mp3
```

For more information, consult `rsid3 --help`.

## Writeable frames

All frames can be printed or deleted, but setting values is only supported for a
subset of text-based frames:

- all simple text (`T***`)
- all simple link (`W***`)
- extended text (`TXXX`)
- extended link (`WXXX`)
- comment (`COMM`)
- lyrics (`USLT`)

More frames may be supported in the future, if deemed useful.
