#!/usr/bin/env bash

IN=$1
OUT=$2
SIZE=$3
PIX_BIN=pixelsortery

function usage() {
    echo "Usage: $0 <IN> <OUT> <NxM> [<OPTIONS>]"
    echo "e.g. $0 in.png out.png 10x10 --right --thres hue:10:40"
    echo
}
function err() {
    echo $@
    exit -1
}

if [[ -z $1 ]]; then
    echo Script to generate a mosaik effect.
    echo "Inspired by this post by u/tododebug : https://www.reddit.com/r/generative/comments/1muvktj/"
    echo
    usage
    exit 0
fi

if [[ -z $1 || -z $2 || -z $3 ]]; then
    usage
    exit -1
fi
# Collect remaining arguments as arguments for the pixelsorter
shift;shift;shift
PIX_ARGS=(${@---vertical})

TMPFOLDER=$(mktemp -d)
: ${TMPFOLDER=/tmp/tmp.mosaik}

tileamount=${SIZE%%x*}
[[ -z $tileamount || ! ( $tileamount -gt 0 ) ]] && err "Wrong syntax: $SIZE"

echo "Splitting image into tiles..."
magick "$IN" -crop "$SIZE"@ +repage +adjoin "$TMPFOLDER/tmp_%05d.png" \
    && (
        for f in "$TMPFOLDER"/*; do
            echo -en "\rsorting $f "
            $PIX_BIN -i "$f" -o "$f" ${PIX_ARGS[@]} || err ""
        done
    ) && echo -e '\nRejoining tiles...'\
    && magick montage -mode concatenate -tile ${tileamount}x "$TMPFOLDER"/* $OUT

rm -r "$TMPFOLDER"

