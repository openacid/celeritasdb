## Intro

this crate is used by celeritasdb and components/XXX as infrastructure to supply
threads functionality.

currently cele_threads wraps `threads-pool` from crate.io which in turn uses
std::thread. and it doesn't use async/await functionality.

## Purpose

this crate is a simple wrapper. the purpose of this crate is to supply common
interfaces to all others and defines the common API interfaces.

when we need to extend functionality or use async/await libary, it is easier to
keep changes only in this crate.
