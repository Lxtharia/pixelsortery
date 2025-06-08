#!/usr/bin/env bash


if ! $(which pixelsortery &> /dev/null ) ; then
    echo "Could not call 'pixelsortery'. Make sure it is installed and included in \$PATH"
    exit -1
fi

declare -a sorteroptions=()

while true ; do
    arg=$1
    case $arg in
        "-i")
            shift
            infile=$1
        ;;
        "-o")
            shift
            outfile=$1
        ;;
        *)
            sorteroptions+=("$1")
        ;;
    esac
    if ! shift ; then break; fi
done


echo Sorting options: ${sorteroptions[@]}
# printf "%s\n" "${sorteroptions[@]}"
echo "=== in: '$infile' | out: '$outfile'"

if [[ -z $infile || -z $outfile ]] ; then
    echo "You need to specify a input and an output file"
    exit -1
fi

tmp1=$(mktemp -d)
tmp2=$(mktemp -d)
tmp_timestamps=$(mktemp)
tmp_framelist=$(mktemp --suffix -framelist.txt)

### Split the video into frames, sort every frame, then stitch all frames back together
### The difficulty is to retain the duration information about a frame
### ffmpeg can read all input files from a text file that includes the frame information though

# Get frame information/duration and save it to a file
echo "=== Getting frame information"
ffprobe -v error -i "$infile" -of csv=p=0 -select_streams v -show_entries packet=pts_time > "$tmp_timestamps"

# Split to frames
echo "=== Splitting video into frames..."
ffmpeg -i "$infile" "$tmp1/frame_%06d.png" || exit -1
    # -progress pipe:1 -loglevel warning 

# Format the file that ffmpeg can read
awk '{
  frame = sprintf("'$tmp2'/frame_%06d.png", NR)
  if (NR > 1) {
    duration = $1 - last
    printf "duration %.6f\n", duration
  }
  print "file '\''" frame "'\''"
  last = $1
} END {
  printf "duration %.6f\n", duration
}' "$tmp_timestamps" > "$tmp_framelist"

echo "=== Sorting frames..."
# Call command once to exit on wrong flags
pixelsortery "${sorteroptions[@]/#}" --fixed 1 -i "$tmp1/frame_000001.png" -o /dev/null || exit -1

# Apply sorting to every frame
for f in "$tmp1/"*.png; do
    framename=$(basename "$f")
    echo "Processing $f ===> $tmp2/$framename"
    cp "$f" "$tmp2/$framename"
    pixelsortery "${sorteroptions[@]/#}" -i "$f" -o "$tmp2/$framename" &
done
wait

echo "=== Creating video..."
# Stitch back together
ffmpeg -f concat -safe 0 -i "$tmp_framelist" -i "$infile" -map 0:v -map 1:a -shortest "$outfile"
    # -c:v libx264 -pix_fmt yuv420p -c:a copy 

# Clean up
rm "$tmp_timestamps" "$tmp_framelist"
rm -r "$tmp1" "$tmp2"


