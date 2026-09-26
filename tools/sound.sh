#!/usr/bin/env bash
# Finish recorded takes into the files the game loads (docs/AUDIO.md, "Workflow").
#
#   tools/sound.sh                  every source under sound/takes/ into assets/audio/
#   THRESHOLD=-40 tools/sound.sh    for noisy recordings: only louder than -40 dB counts as sound
#
# sound/takes/sfx/NAME.{flac,wav,ogg}: takes separated by at least 1 s of silence become
# NAME_1.ogg, NAME_2.ogg ... (a single take: NAME.ogg), each trimmed, peak -1 dBFS, faded, mono
# 48 kHz. Loops and sound/takes/music/ are only levelled and encoded. Needs SoX (brew install sox)
# and ffmpeg.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
TAKES=$ROOT/sound/takes
OUT=$ROOT/assets/audio
THRESHOLD=${THRESHOLD:--48}
LOOPS=" torpedo_run "
MUSIC=" build wave "
MAX_VARIANTS=8
LONGEST=4
GAP=1
KEYS=" $(sed -nE 's/.*spec\("([a-z0-9_]+)".*/\1/p' "$ROOT/src/game/audio.rs" | tr '\n' ' ')torpedo_run "

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
written=" "

warn() { echo "  warning: $*" >&2; }

names() {
    ls "$1" 2>/dev/null | sed -nE 's/\.(flac|wav|ogg)$//p' | sort -u
}

# The preferred source for a name: a fresh FLAC export beats the bootstrap OGG.
source_for() {
    local found="" f ext
    for ext in flac wav ogg; do
        f=$1/$2.$ext
        [ -f "$f" ] || continue
        if [ -z "$found" ]; then found=$f; else warn "$(basename "$f") ignored, $(basename "$found") is used"; fi
    done
    echo "$found"
}

emit() {
    local seconds
    seconds=$(soxi -D "$1")
    written="$written$1 "
    printf '  %-24s %6.2f s\n' "$(basename "$1")" "$seconds"
    if [ "$2" = check ] && awk -v s="$seconds" -v l="$LONGEST" 'BEGIN { exit !(s > l) }'; then
        warn "$(basename "$1") is longer than $LONGEST s: are two takes closer than $GAP s?"
    fi
}

# "start end" in seconds for every stretch of sound in $1 between silences of at least $GAP s.
sounds_in() {
    ffmpeg -hide_banner -nostats -i "$1" -af "silencedetect=n=${THRESHOLD}dB:d=$GAP" -f null - 2>&1 |
        awk -v total="$(soxi -D "$1")" '
            BEGIN { start = 0 }
            /silence_start:/ { if ($NF > start) print start, $NF; start = -1 }
            /silence_end:/ { for (i = 1; i < NF; i++) if ($i == "silence_end:") start = $(i + 1) }
            END { if (start >= 0 && total - start >= 0.02) print start, total }'
}

one_shots() {
    local src=$1 name=$2 n i out from to
    sox "$src" "$tmp/whole.wav" remix - rate 48k norm -1
    sounds_in "$tmp/whole.wav" > "$tmp/sounds"
    n=$(wc -l < "$tmp/sounds" | tr -d ' ')
    [ "$n" -gt 0 ] || { warn "$name: nothing louder than $THRESHOLD dB"; return; }
    [ "$n" -le "$MAX_VARIANTS" ] || warn "$name: $n takes, the game plays the first $MAX_VARIANTS"
    i=0
    while read -r from to; do
        i=$((i + 1))
        [ "$i" -le "$MAX_VARIANTS" ] || break
        out=$OUT/sfx/${name}_$i.ogg
        [ "$n" -gt 1 ] || out=$OUT/sfx/$name.ogg
        sox "$tmp/whole.wav" -C 5 "$out" trim "$from" "=$to" \
            silence 1 0.005 "${THRESHOLD}d" fade 0.002 \
            reverse silence 1 0.005 "${THRESHOLD}d" fade 0.02 reverse norm -1
        emit "$out" check
    done < "$tmp/sounds"
}

mkdir -p "$OUT/sfx" "$OUT/music"

echo "sfx"
for name in $(names "$TAKES/sfx"); do
    case $KEYS in *" $name "*) ;; *) warn "$name is not a sound in src/game/audio.rs" ;; esac
    src=$(source_for "$TAKES/sfx" "$name")
    rm -f "$OUT/sfx/$name.ogg" "$OUT/sfx/${name}_"[1-9].ogg
    case $LOOPS in
        *" $name "*)
            sox "$src" -C 5 "$OUT/sfx/$name.ogg" remix - rate 48k norm -1
            emit "$OUT/sfx/$name.ogg" loop ;;
        *) one_shots "$src" "$name" ;;
    esac
done

echo "music"
for name in $(names "$TAKES/music"); do
    case $MUSIC in *" $name "*) ;; *) warn "$name is not a music layer the game plays" ;; esac
    src=$(source_for "$TAKES/music" "$name")
    sox "$src" -C 5 "$OUT/music/$name.ogg" channels 2 rate 48k norm -1
    emit "$OUT/music/$name.ogg" loop
done

for f in "$OUT"/sfx/*.ogg "$OUT"/music/*.ogg; do
    [ -e "$f" ] || continue
    case $written in *" $f "*) ;; *) warn "${f#"$ROOT"/} has no source in sound/takes/" ;; esac
done

python3 "$ROOT/tools/art_fit.py" check
