#!/bin/bash

#### Default Current Machine
function current() {
    cargo build --release

    ## check now os.
}
function init_android() {
    cargo install cargo-ndk
    
    rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android

}
#### Android ####
function android() {
    
    cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 -o ./jniLibs build --release 

    ## crate & link to android directory
    mkdir -p core/android/src/main/jniLibs/arm64-v8a
    mkdir -p core/android/src/main/jniLibs/armeabi-v7a
    mkdir -p core/android/src/main/jniLibs/x86
    echo 'android jniLibs directory build ok!'

    cp -rf ./jniLibs/ core/android/src/main/jniLibs/

    echo 'Flutter: Android dynamic library is ok!'
}

#### IOS ####
function ios() {
    cargo lipo --release
    echo 'Rust: Ios release build ok!'
    cp target/universal/release/libesse.a core/ios/share/libesse.a
    echo 'Flutter: Ios dynamic library is ok!'
}

#### Linux ####
function linux() {
    cargo build --release ### there maybe not use in other linux distribution.
    echo 'Rust: Linux release build ok!'
    cp target/release/libesse.a core/linux/share/libesse.a
    echo 'Flutter: Linux dynamic library is ok!'
}

#### MacOS ####
function macos() {
    cargo build --release
    echo 'Rust: Macos release build ok!'
    cp target/release/libesse.a core/macos/share/libesse.a
    echo 'Flutter: Macos dynamic library is ok!'
}

#### Windows ####
function windows() {
    cargo build --release ### there maybe not use in other windows distribution.
    echo 'Rust: windows release build ok!'
    cp target/release/esse.dll core/windows/share/esse.dll
    cp target/release/esse.dll.lib core/windows/share/esse.dll.lib
    echo 'Flutter: windows dynamic library is ok!'
}

#### Web ####
function web() {
    echo 'WIP'
}

if [ $# -eq 0 ]
then
    echo "Usage: ./rust.sh [OPTION]"
    echo "Rust dynamic library auto generator."
    echo ""
    echo "OPTION:"
    echo "  current    build current machine's library."
    echo "  all        build all versions libraries."
    echo "  android    only build for Android."
    echo "  ios        only build for IOS."
    echo "  linux      only build for Linux. (Linux Machine)"
    echo "  macos      only build for MacOS. (MacOS Machine)"
    echo "  windows    only build for Windows. (Windows Machine)"
    echo "  web        only build for web (wasm)."
else
    echo "Now is building: $1"
    $1
fi

