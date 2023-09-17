run: build-examples
  cargo run --package milliservices_core

http: build-examples
  cargo run --package milliservices_http_layer;

build: build-examples
  echo -e "\n:::::::::: building core ::::::::::\n";
  cargo build

mem p:
  grep -E '^Vm(Peak|Size)' /proc/{{p}}/status

test: build-fixtures
  cargo test --test='test*'

bench:
  wrk -t16 -c100 -d30 http://localhost:3000/rust-final -s wrk.lua

build-examples:
  #!/usr/bin/env sh
  for dir in `find ./examples/* -type f -name justfile | xargs dirname`; do
    echo -e "\n:::::::::: building $dir ::::::::::\n";
    just -d "$dir" -f "$dir/justfile" build || exit 1;
  done

build-fixtures:
  #!/usr/bin/env sh
  for dir in `find ./test-fixtures/* -type f -name justfile | xargs dirname`; do
    just -d "$dir" -f "$dir/justfile" build || exit 1;
  done

fix:
  cargo fix --allow-dirty --allow-staged
  cargo fmt --all

pkg path *args:
  just -d "packages/{{path}}" -f "packages/{{path}}/justfile" {{args}}

core *args:
  just pkg core {{args}}

