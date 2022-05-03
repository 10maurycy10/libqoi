# libqoi

[![crates.io badge](https://img.shields.io/crates/v/libqoi)](https://crates.io/crates/libqoi) [![docs.rs badge](https://img.shields.io/docsrs/libqoi)](https://docs.rs/libqoi/latest/libqoi/)

A basic rust [QOI](https://qoiformat.org/) decoder/encoder.

## Why QOI

QOI is a lossless image format with a [one page specification](https://qoiformat.org/qoi-specification.pdf). It can achieve better compression than PNG, while being much faster than PNG.

The best possible time complexity is O(n) where n is the amount of pixels, and space is O(1). This encoder has O(n) time and O(n) space complexity

## Demo

This cat photo from https://commons.wikimedia.org/wiki/File:Cat_poster_1.jpg (5935 × 3898) is 29,291,338 bytes as PNG but 27,960,953 as QOI.

QOI also takes >.2 seconds to compress and decompress (on my machine), but PNG takes one second just to compress.
