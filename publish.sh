#! /bin/zsh
./build_web.sh
cd pkg
vercel --prod -c
cd ../
