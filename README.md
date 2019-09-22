# gene-rs [![Build Status](https://travis-ci.com/gcao/gene.rs.svg?branch=master)](https://travis-ci.com/gcao/gene.rs)

## Introduction

Gene is a pet language I'm building. It is written in Gene data format. There is Gene data format, then there is Gene language, not like JS vs JSON.

## MISC

<pre>while 1; do fswatch -v -r src tests Cargo.toml | cargo test; sleep 0.2; done</pre>

<pre>while 1; do fswatch -v -r src tests Cargo.toml | cargo test --features wip_tests test_wip; sleep 0.2; done</pre>

## License

MIT

## CREDITS

https://github.com/utkarshkukreti/edn.rs
