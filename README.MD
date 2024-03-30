# rsid3

A simple, command line ID3v2 tag editor designed for scripting.

## Features

- Print all frames of a file,
- Print, set or delete chosen frames,
- Convert between ID3v2.2, ID3v2.3 and ID3v2.4 versions,
- Purge all tags belonging to a specific (or any) ID3v2 version.

## Examples

```sh
# If no actions are chosen, print a summary of a file with all frames
rsid3 file.mp3

# Action 1: Printing frames
rsid3 --TIT2 file.mp3                        # Print TIT2 (title)
rsid3 --TXXX some_desc file.mp3              # Print TXXX[some_desc]
rsid3 --COMM some_desc eng file.mp3          # Print COMM[some_desc](eng)
rsid3 --COMM some_desc first file.mp3        # Print COMM[some_desc] and any first language found
rsid3 -d, --TIT2 --TPE1 --TALB file.mp3      # Print title, artist and album, delimited by comma

# Action 2: Setting frames
rsid3 --TIT2= 'My Title' file.mp3                  # Set TIT2
rsid3 --TXXX= some_desc 'some value' file.mp3      # Set TXXX[some_desc]
rsid3 --COMM= some_desc eng 'My comment' file.mp3  # Set COMM[some_desc](eng)

# Action 3: Deleting frames
rsid3 --TIT2- file.mp3                   # Delete TIT2
rsid3 --TXXX- some_desc file.mp3         # Delete TXXX[some_desc]
rsid3 --COMM- some_desc eng file.mp3     # Delete COMM[some_desc](eng)

# Action 4: Converting between ID3v2 versions
rsid3 --id3v2.4 file.mp3                 # Try losslessly converting a tag to ID3v2.4
rsid3 --force-id3v2.2 file.mp3           # Forcefully convert a tag to ID3v2.2
rsid3 --force-id3v2.2 \
      --force-id3v2.3 \
      --force-id3v2.4 file.mp3           # Leave only frames compatible with all three ID3v2
                                         # standards, and store as ID3v2.4

# Action 5: Purging frames
rsid3 --purge-id3v2.3 file.mp3           # Remove all tag information if it's ID3v2.3
rsid3 --purge-all file.mp3               # Remove all tag information, irrespective of version

# All actions can be chained and mixed; they are executed in the argument order
rsid3 --TIT2 --TIT2= new_title file.mp3             # Print old title and set new title
rsid3 --id3v2.3 --TPE1= new_artist --TALB file.mp3  # Try to convert to ID3v2.3, set artist
                                                    # and print album

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
