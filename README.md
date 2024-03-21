# wavrw

`wavrw` is a command line tool to read (and someday write) WAV files with a focus on visualizing the structure of files and parsing metadata.

It's initially intended as a learning project with aspirations to be generally useful. It's currently in *very* early days and may not be useful to anyone else. 

Current status: Most chunks from the initial WAV spec [RIFF1991](https://wavref.til.cafe/spec/riff1991/) are supported. Only supports reading metadata. Doesn't yet support writing metadata. Doesn't yet read or write audio data (samples). 

I intend to evntually support every WAV chunk I can find a sample of. If there's something you'd like supported, please let me know by adding [an issue](https://github.com/briandorsey/wavrw/issues). If you have them, please include a link to a specification and one or more example files.

Take care,
-Brian

# Details

`wavrw` answers questions about WAV files: 
  * What is *actually* in this file?
  * Does another program preserve all WAV chunks when saving and exporting files? (by running it before and after the other program and comparing the output)

## Example output

```
$ wavrw view test_wavs/example_a.wav`
test_wavs/example_a.wav: 
      offset id              size summary
          12 fmt               16 PCM (0x1), 1 chan, 24/48000
          36 bext             604 BWDate, BWTime, BWDescription
         648 data            1440 audio data
        2096 LIST-adtl         70 labl(3)
        2174 ID3             2048 ...
        4230 SMED            8812 ...
       13050 LIST-INFO        214 IPRD, IGNR, ISFT, INAM, IARL, ICOP, IART, ICMT
       13272 iXML            4516 ...
       17796 cue               76 3 cue points
       17880 _PMX            3706 ...
       21594 MD5               16 0x37A5BED4393B8F3708963F5E59C7F483
       21618 CSET               8 code_page: (0), United States of America(1), En ...
```

## Example detailed output

```
$ wavrw view --format=detailed test_wavs/example_a.wav
test_wavs/example_a.wav: 
      offset id              size summary
          12 fmt               16 PCM (0x1), 1 chan, 24/48000
             |             format_tag : WAVE_FORMAT_PCM (0x1)
             |               channels : 1
             |        samples_per_sec : 48000
             |      avg_bytes_per_sec : 144000
             |            block_align : 3
             |        bits_per_sample : 24
             --------------------------------------
          36 bext             604 
             |            description : BWDescription
             |             originator : BWOriginator
             |   originator_reference : BWOriginatorRef
             |       origination_date : BWDate
             |       origination_time : BWTime
             |         time_reference : 0
             |                version : 1
             |                   umid : 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
             |         loudness_value : 0
             |         loudness_range : 0
             |    max_true_peak_level : 0
             | max_momentary_loudness : 0
             |max_short_term_loudness : 0
             |         coding_history : 22
             --------------------------------------
         648 data            1440 audio data
        2096 LIST-adtl         70 labl(3)
             |                   labl :   1, Region 01
             |                   labl :   2, Marker 01
             |                   labl :   3, Marker 02
             --------------------------------------
        2174 ID3             2048 ...
        4230 SMED            8812 ...
       13050 LIST-INFO        214 chunk: text
             |                   IPRD : CDTitle
             |                   IGNR : Category
             |                   ISFT : Soundminer
             |                   INAM : TrackTitle
             |                   IARL : BWOriginator
             |                   ICOP : TrackYear Manufacturer (Library) URL
             |                   IART : Artist
             |                   ICMT : Description
             --------------------------------------
       13272 iXML            4516 ...
       17796 cue               76 name: position, chunk_id, chunk_start, block_start, sample_offset
             |                      1 :          0, data,          0,          0,          0
             |                      2 :        240, data,          0,          0,        240
             |                      3 :        360, data,          0,          0,        360
             --------------------------------------
       17880 _PMX            3706 ...
       21594 MD5               16 0x37A5BED4393B8F3708963F5E59C7F483
       21618 CSET               8 code_page: (0), United States of America(1), English(9), US(1)
             |              code_page : 0
             |           country_code : United States of America(1)
             |               language : English(9)
             |                dialect : US(1)
             --------------------------------------
```

## Help overview

```
WAV file metadata read/write utility

Usage: wavrw <COMMAND>

Commands:
  view   Summarize WAV file structure and metadata
  list   List directories of files, show single line summary of chunks
  topic  Print additional help and reference topics
  help   Print this message or the help of the given subcommand(s)

Global Options:
  -h, --help     Print help
  -V, --version  Print version

```
