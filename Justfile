project := "serde_sjson"
default := "run"

build *ARGS:
    cargo build {{ARGS}}
    cargo readme > README.md

run *ARGS:
    cargo run -- {{ARGS}}

test *ARGS:
    cargo test {{ARGS}}

check:
    cargo clippy -- -D warnings
    cargo test

doc:
    cargo doc --no-deps
    cargo readme > README.md

serve-doc port='8000': doc
    python3 -m http.server {{port}} --directory target/doc

release version execute='': check build doc
    git fetch --all
    [ "$(git rev-parse master)" = "$(git rev-parse origin/master)" ] \
        || (echo "error: master and origin/master differ" >&2; exit 1)
    cargo release --sign --allow-branch master {{ if execute != "" { '-x' } else { '' } }} {{version}}

coverage *ARGS:
    RUSTFLAGS="-C instrument-coverage" cargo test --tests {{ARGS}} || true
    cargo profdata -- merge -sparse default*.profraw -o {{project}}.profdata
    rm default*.profraw

cov-report *ARGS:
    #!/bin/bash
    RUSTFLAGS="-C instrument-coverage" cargo test --tests {{ARGS}} || true
    cargo profdata -- merge -sparse default*.profraw -o {{project}}.profdata
    rm default*.profraw

    cargo cov -- report \
        $(for file in \
            $(RUSTFLAGS="-C instrument-coverage" cargo test --tests --no-run --message-format=json\
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
            ); \
          do \
            printf "%s %s " -object $file; \
          done \
        ) \
    --use-color --ignore-filename-regex='/.cargo/registry' \
    --instr-profile={{project}}.profdata --summary-only

cov-show *ARGS:
    #!/bin/bash
    RUSTFLAGS="-C instrument-coverage" cargo test --tests {{ARGS}} || true
    cargo profdata -- merge -sparse default*.profraw -o {{project}}.profdata
    rm default*.profraw

    cargo cov -- show \
        $(for file in \
            $(RUSTFLAGS="-C instrument-coverage" cargo test --tests --no-run --message-format=json\
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
            ); \
          do \
            printf "%s %s " -object $file; \
          done \
        ) \
    --use-color --ignore-filename-regex='/.cargo/registry' \
    --instr-profile={{project}}.profdata \
    --show-instantiations --show-line-counts-or-regions \
    --Xdemangler=rustfilt | bat
