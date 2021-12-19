#! /bin/zsh
rm -rf pkg
wasm-pack build --target web --release
cp -r static/*(D) pkg/
cp -r assets pkg/assets
