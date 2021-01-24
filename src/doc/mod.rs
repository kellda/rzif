/*!
Various documentation

Hexadecimal numbers are written with an initial dollar, as in $ff, while binary numbers are written with a double-dollar as in $$11011.

# Fonts
The fonts are:
1. the normal font
2. don't exists: must be refused
3. a [character graphics font](http://inform-fiction.org/zmachine/standards/z1point1/sect16.html)
4. a Courier-style font with fixed pitch

# Colors
## True colors
True colors are 16 bits colors in the fromat `$$0bbbbbgggggrrrrr` (a 0 bit followed by 5 bits for blue, 5 for green and 5 for red).
There are two specials cases:
* -1: default color
* -2: current color

## Color codes
Color codes to use with [`Config.default_color`](crate::Config#structfield.default_color).
```text
0 = current (true -2)
1 = default (true -1)
2 = black   (true $0000, $$0000000000000000)
3 = red     (true $001D, $$0000000000011101)
4 = green   (true $0340, $$0000001101000000)
5 = yellow  (true $03BD, $$0000001110111101)
6 = blue    (true $59A0, $$0101100110100000)
7 = magenta (true $7C1F, $$0111110000011111)
8 = cyan    (true $77A0, $$0111011110100000)
9 = white   (true $7FFF, $$0111111111111111)
```

# Starting a game
## In Versions 1 to 4
The screen should be cleared and the text cursor placed at the bottom left (so that text scrolls upwards as the game gets under way).

## In Version 5 and later
The screen should be cleared and the text cursor placed at the top left.

> Note: the version is the first byte of the story file.

# Windows
The screen is divided into a lower and an upper window and at any given time one of these is selected. (Initially it is the lower window.) The game uses the set_window opcode to select one of the two. Each window has its own cursor position at which text is printed. Operations in the upper window do not move the cursor of the lower. Whenever the upper window is selected, its cursor position is reset to the top left. Selecting, or re-sizing, the upper window does not change the screen's appearance.

An interpreter should use a fixed-pitch font when printing on the upper window.

Printing onto the upper window overlays whatever text is already there.

When text reaches the bottom right of the lower window, it should be scrolled upwards. (When the text style is Reverse Video the new blank line should not have reversed colours.) The upper window should never be scrolled: it is legal for a character to be printed on the bottom right position of the upper window (but the position of the cursor after this operation is undefined: the author suggests that it stay put).

## Cursor
In Version 1 to 4, the lower window's cursor is always on the bottom screen line. In Version 5 it can be at any line which is not underneath the upper window. If a split takes place which would cause the upper window to swallow the lower window's cursor position, the interpreter should move the lower window's cursor down to the line just below the upper window's new size.

## Spliting
The upper window has variable height (of n lines) and the same width as the screen. This should be displayed on the n lines of the screen (in Vesion 3, below the top one which continues to hold the status line). Initially the upper window has height 0. When the lower window is selected, the game can split off an upper window of any chosen size by using the split_window opcode.

# Styles
Sets the text style to: Roman (if 0), Reverse Video (if 1), Bold (if 2), Italic (4), Fixed Pitch (8). In some interpreters (though this is not required) a combination of styles is possible (such as reverse video and bold). In these, changing to Roman should turn off all the other styles currently set.

It is legal to request style combinations in a single set_text_style opcode by adding the values (which are powers of two) together. If the parameter is non-zero, then all the styles given are activated. If the parameter is zero, then all styles are deactivated. If the interpreter is unable to provide the requested style combination, it must give precedence first to the styles requested in the most recent call to set_text_style, and within that to the highest bit, making the priority Fixed, Italic, Bold, Reverse.

An interpreter need not provide Bold or Italic (even for font 1) and is free to interpret them broadly. (For example, rendering bold-face by changing the colour, or rendering italic with underlining.)

# Buffering
Text printing may be "buffered" in that new-lines are automatically printed to ensure that no word (of length less than the width of the screen) spreads across two lines (if the interpreter is able to control this). (This process is sometimes called "word-wrapping".)

Buffering is on by default (at the start of a game) and a game can switch it on or off using the buffer_mode opcode. Text buffering is never active in the upper window.

If set to 1, text output on the lower window in stream 1 is buffered up so that it can be word-wrapped properly. If set to 0, it isn't.

# Erasing
## Erasing a window
Using the opcode erase_window, the specified window can be cleared to background colour. (Even if the text style is Reverse Video the new blank space should not have reversed colours.)

In Versions 5 and later, the cursor for the window being erased should be moved to the top left. In Version 4, the lower window's cursor moves to its bottom left, while the upper window's cursor moves to top left.

Erasing window -1 clears the whole screen to the background colour of the lower screen, collapses the upper window to height 0, moves the cursor of the lower screen to bottom left (in Version 4) or top left (in Versions 5 and later) and selects the lower screen. The same operation should happen at the start of a game.

## Erasing a line
Using erase_line in the upper window should erase the current line from the cursor position to the right-hand edge, clearing it to background colour. (Even if the text style is Reverse Video the new blank space should not have reversed colours.)

# Bleeps
This interpreter support only one sound effect: a beep or bell sound, which we shall call a "bleep".
Bleep number 1 is a high-pitched bleep, number 2 a low-pitched one

# Files
## Transcript
The transcript contains the game output and the player inputs. If an interpreter decides where to send the transcript by asking the player for a filename, this question should only be asked once per game session.

## Command file
The command file contains all the player inputs.

## Save
A save file contains the state of the game. It is encoded in [Quetzal](http://inform-fiction.org/zmachine/standards/quetzal/index.html) format in this interpreter.

# Text
**TODO: arrows, fn, numeric**

The interpreter is required to be able to print representations of every defined Unicode character under $0100 (i.e. of every defined ISO 8859-1 Latin1 character). If no suitable letter forms are available, textual equivalents may be used.

An interpreter is not required to have suitable letter-forms for printing Unicode characters $0100 to $FFFF. (It may, if it chooses, allow the user to configure certain fonts for certain Unicode ranges; but this is not required.) If a Unicode character must be printed which an interpreter has no letter-form for, a question mark should be printed instead.

## Unicode characters
|Unicode code (hex)|Name|Character|Textual Equivalent|
|---|-----------|-|--|
|0e4|a-diaeresis|ä|ae|
|0f6|o-diaeresis|ö|oe|
|0fc|u-diaeresis|ü|ue|
|0c4|A-diaeresis|Ä|Ae|
|0d6|O-diaeresis|Ö|Oe|
|0dc|U-diaeresis|Ü|Ue|
|0df|sz-ligature|ß|ss|
|0bb|quotation|»|>> or "|
|0ab|marks|«|<< or "|
|0eb|e-diaeresis|ë|e|
|0ef|i-diaeresis|ï|i|
|0ff|y-diaeresis|ÿ|y|
|0cb|E-diaeresis|Ë|E|
|0cf|I-diaeresis|Ï|I|
|0e1|a-acute|á|a|
|0e9|e-acute|é|e|
|0ed|i-acute|í|i|
|0f3|o-acute|ó|o|
|0fa|u-acute|ú|u|
|0fd|y-acute|ý|y|
|0c1|A-acute|Á|A|
|0c9|E-acute|É|E|
|0cd|I-acute|Í|I|
|0d3|O-acute|Ó|O|
|0da|U-acute|Ú|U|
|0dd|Y-acute|Ý|Y|
|0e0|a-grave|à|a|
|0e8|e-grave|è|e|
|0ec|i-grave|ì|i|
|0f2|o-grave|ò|o|
|0f9|u-grave|ù|u|
|0c0|A-grave|À|A|
|0c8|E-grave|È|E|
|0cc|I-grave|Ì|I|
|0d2|O-grave|Ò|O|
|0d9|U-grave|Ù|U|
|0e2|a-circumflex|â|a|
|0ea|e-circumflex|ê|e|
|0ee|i-circumflex|î|i|
|0f4|o-circumflex|ô|o|
|0fb|u-circumflex|û|u|
|0c2|A-circumflex|Â|A|
|0ca|E-circumflex|Ê|E|
|0ce|I-circumflex|Î|I|
|0d4|O-circumflex|Ô|O|
|0db|U-circumflex|Û|U|
|0e5|a-ring|å|a|
|0c5|A-ring|Å|A|
|0f8|o-slash|ø|o|
|0d8|O-slash|Ø|O|
|0e3|a-tilde|ã|a|
|0f1|n-tilde|ñ|n|
|0f5|o-tilde|õ|o|
|0c3|A-tilde|Ã|A|
|0d1|N-tilde|Ñ|N|
|0d5|O-tilde|Õ|O|
|0e6|ae-ligature|æ|ae|
|0c6|AE-ligature|Æ|AE|
|0e7|c-cedilla|ç|c|
|0c7|C-cedilla|Ç|C|
|0fe|Icelandic thorn|þ|th|
|0f0|Icelandic eth|ð|th|
|0de|Icelandic Thorn|Þ|Th|
|0d0|Icelandic Eth|Ð|Th|
|0a3|pound symbol|£|L|
|153|oe-ligature|œ|oe|
|152|OE-ligature|Œ|OE|
|0a1|inverted !|¡|!|
|0bf|inverted ?|¿|?|

# Sources
[Output streams and file handling](http://inform-fiction.org/zmachine/standards/z1point1/sect07.html)\
[The screen model](http://inform-fiction.org/zmachine/standards/z1point1/sect08.html)\
[Sound effects](http://inform-fiction.org/zmachine/standards/z1point1/sect09.html)\
[Input streams and devices](http://inform-fiction.org/zmachine/standards/z1point1/sect10.html)\
[Dictionary of opcodes](http://inform-fiction.org/zmachine/standards/z1point1/sect15.html)\
[How text and characters are encoded](http://inform-fiction.org/zmachine/standards/z1point1/sect03.html)
*/
