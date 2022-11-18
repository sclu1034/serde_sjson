project := "serde_sjson"
default := "run"

run *ARGS:
    cargo run -- {{ARGS}}

test *ARGS:
    cargo test {{ARGS}}

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