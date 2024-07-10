#!/bin/bash
cargo publish --registry tataku-registry -p tataku-common-proc-macros --allow-dirty
cargo publish --registry tataku-registry -p tataku-common --allow-dirty