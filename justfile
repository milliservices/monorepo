run: build-packages
  cargo run --package milliservices_core

watch:
  cargo watch --shell 'just run'

pkg path *args:
  just -d "packages/{{path}}" -f "packages/{{path}}/justfile" {{args}}

core *args:
  pkg core {{args}}

build-packages:
  #!/usr/bin/env sh
  for dir in `find ./packages/* -type f -name justfile | xargs dirname`; do
    echo "Building $dir";
    just -d "$dir" -f "$dir/justfile" build;
  done

fix:
  cargo fix --allow-staged
  cargo fmt --all

