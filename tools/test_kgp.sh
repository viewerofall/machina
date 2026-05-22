#!/usr/bin/env bash
# Minimal kitty graphics protocol test. If you don't see an icon below,
# then KGP unicode placeholders aren't working in this kitty session.
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SPRITE="$SCRIPT_DIR/../assets/icons/rs.png"

if [[ ! -f "$SPRITE" ]]; then
    echo "missing $SPRITE — run gen_icons.py first" >&2
    exit 1
fi

# Transmit the rust sprite as image id 1, virtual (U=1), format PNG (f=100)
B64=$(base64 -w0 "$SPRITE")
LEN=${#B64}
CHUNK=4096

echo "transmitting sprite (id=1)..."
i=0
while [[ $i -lt $LEN ]]; do
    chunk=${B64:$i:$CHUNK}
    end=$((i + CHUNK))
    if [[ $end -ge $LEN ]]; then
        m=0
    else
        m=1
    fi
    if [[ $i -eq 0 ]]; then
        printf '\e_Ga=t,U=1,i=1,f=100,t=d,m=%s;%s\e\\' "$m" "$chunk"
    else
        printf '\e_Gm=%s;%s\e\\' "$m" "$chunk"
    fi
    i=$end
done

# Read kitty's response so it doesn't pollute the next prompt
sleep 0.1

echo ""
echo "next line should show a rust icon followed by 'rust file':"
# placeholder char + fg color set to image id 1 (RGB 0,0,1)
printf '\e[38;2;0;0;1m\xf4\x8e\xbb\xae\xcc\x85\xcc\x85\e[39m rust file\n'
echo ""
echo "without diacritics:"
printf '\e[38;2;0;0;1m\xf4\x8e\xbb\xae\e[39m rust file\n'
echo ""
echo "if you see icons above, KGP works. if not, the protocol output is wrong."
