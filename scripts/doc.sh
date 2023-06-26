#!/bin/bash

sed "/^\/\/\!/d" -i src/lib.rs

IFS=''
echo "//!" > tmp.txt
while read line; do
    echo "//!$line" >> tmp.txt
done < README.md
echo "//!" >> tmp.txt
cat src/lib.rs >> tmp.txt

mv -fv tmp.txt src/lib.rs
