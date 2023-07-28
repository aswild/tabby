# tabby

List the contents of small files in a table. This is mostly useful for listing attributes in /sys or
/proc, like a fancy version of `for f in *; do echo "${f}: $(cat "$f")"; done` but with cleaner
formatting.
