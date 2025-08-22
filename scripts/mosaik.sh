#!/usr/bin/env bash

IN=$1
OUT=$2
SIZE=$3
PIX_BIN=pixelsortery

function usage() {
    echo "Usage: $0 <in> <out> 10x10"
    echo
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

function err() {
    echo $@
    exit
}

TMPFOLDER=$(mktemp -d)
: ${TMPFOLDER=/tmp/tmp.mosaik}

tileamount=${SIZE%%x*}
[[ -z $tileamount || ! ( $tileamount -gt 0 ) ]] && err "Wrong syntax: $SIZE"

magick "$IN" -crop "$SIZE"@ +repage +adjoin "$TMPFOLDER/tmp_%05d.png"  &&\
    (
        for f in "$TMPFOLDER"/*; do
            echo -en "sorting $f\r"
            $PIX_BIN "$f" --vertical "$f"
        done
    ) && echo -e '\nRejoining tiles...' &&\
    magick montage -mode concatenate -tile ${tileamount}x "$TMPFOLDER"/* $OUT

rm -r "$TMPFOLDER"

