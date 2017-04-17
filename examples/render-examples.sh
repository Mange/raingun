#!/bin/bash

examples_dir="$(dirname "$0")"
project_dir="$(readlink -f "${examples_dir}/..")"
bin="${project_dir}/target/release/raingun"

cargo build --release
for example in "${examples_dir}"/*.yml; do
  "$bin" "$example"
done
