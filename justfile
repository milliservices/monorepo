run: build-examples
  cargo run --package milliservices_core

build: build-examples
  echo -e "\n:::::::::: building core ::::::::::\n";
  cargo build

test: build-fixtures
  cargo test --test='test*'

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

