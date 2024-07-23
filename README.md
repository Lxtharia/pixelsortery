# Pixelsortery - A Pixelsorter Tool written in Rust!

Inspired by [ASDFPixelSort](https://github.com/kimasendorf/ASDFPixelSort) by [Kim Asendorf](https://kimasendorf.com/) I've written a few Pixelsorters before.

My first one originated from modifying the original processing code until it became a little Java Swing Application: [Pixel-Sorter-App](https://github.com/Lxtharia/pixel-sorter-app).

My second implementation was in C. It may or may not have been faster but it's usable from the command line!

But now I have _engineered THIS pixelsorter_ with _far greater goals_, created as a rust practice project, striving to be THE BEST and FASTEST FOSS-Pixelsorter of them all!!!

## Installation
```
git clone https://github.com/Lxtharia/pixelsortery-rs
cd pixelsortery-rs
cargo install
```

## Usage (CLI)
```
pixelsorter <input> <output> [options]

pixelsorter --help
```
Just experiment with it and read the explanation below, you'll get it! :)

## Explanation (How it works)

This is a very MODULAR sorter.
There are multiple stages to allow for easy (future) customization:

1. **Iterating**
    - The sorting direction, but can be much more
    - Allows to traverse the pixels in reverse
    - Allows to sort in vertical or horizontal lines
    - But also in different shapes
2. **Selecting**
    - The magic behind the cool pixelsort-effect: we only sort _intervals_ of pixels (calling them spans)
    - We are iterating pixels in whatever order the iteration has yielded to us
    - We select these intervals by grouping similar pixels together
    - Examples:
        - Group (select) bright pixels (pixels with their brightness value between 180 and 255)
        - Select blue pixels (pixels with their hue value between 190 and 240)
    - All other pixels, not fulfilling this requirement, will not get sorted
    - We only group pixels that come after each other, so (D, B, B, D, D, B, D) gives us the spans (B, B) and (B) 
        - With B being bright and D being dark, following the previous example
3. **Sorting**
    - Finally, sorting
    - Can sort by different criteria
    - Brightness looks the most smooth
    - Also allows to change the sorting algorithm, which can create different results (see [Sorting algorithm stability](https://en.wikipedia.org/wiki/Sorting_algorithm#Stability))

- I will create such a great visualisation graphic at some point and add it here (it's gonna be so good)

## Features

- [x] Choose to sort lines in all 4 directions
- [x] Choose to select spans of random length
- [x] Choose to select spans based on their hue/brightness/saturation
- [x] Sorts pixel with MAPSORT, HELL YEAH!!
- [x] Also allows to sort pixels COMPLETELY BROKEN creating a super rad glitch effect
- [x] A Super Cool CLI interface! (it's ok)

## Planned Features

-[ ] a GUI (holy damn)
-[ ] a really good GUI (damn holy)
-[ ] a godlike GUI where you can draw in real time (god damn holy jesus)
-[ ] Sort by more advanced patterns (Round/square spiral, diagonally by any angle, hilbert curve)
-[ ] Allow to read iteration data from file
-[ ] Allow to use MASKS
-[ ] _Combine_ masks with the normal
-[ ] Allow to combine selectors (sort pixels that are bright _and_ red)
-[ ] Sort _multiple_ times (With same selectors)
-[ ] Internal chain sorting (low prio)
-[ ] Internal sorting animation (low prio)

## Yeah
yeah
```
Yeah!!
```
> yeah

| yeah |
| ---- |
| _yeah_ |
