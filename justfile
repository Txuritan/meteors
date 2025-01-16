build-android $CARGO_NET_GIT_FETCH_WITH_CLI="true":
    cross +nightly build -Z build-std=panic_abort,std --release --target aarch64-linux-android

build-android-debug $CARGO_NET_GIT_FETCH_WITH_CLI="true":
    cross +nightly build --target aarch64-linux-android

build-linux $CARGO_NET_GIT_FETCH_WITH_CLI="true":
    cargo build -Z build-std=panic_abort,std --release --target x86_64-unknown-linux-gnu

build-linux-bloat $CARGO_NET_GIT_FETCH_WITH_CLI="true":
    cargo bloat -Z build-std=panic_abort,std --release --target x86_64-unknown-linux-gnu

build-windows $CARGO_NET_GIT_FETCH_WITH_CLI="true":
    cargo build -Z build-std=panic_abort,std --release --target x86_64-pc-windows-msvc

build-windows-bloat $CARGO_NET_GIT_FETCH_WITH_CLI="true":
    cargo bloat -Z build-std=panic_abort,std --release --target x86_64-pc-windows-msvc

build-tailwind:
    npx tailwindcss -c tailwind.config.js -i crates/varela-command-serve/assets/index.css -o crates/varela-command-serve/assets/dist/index.css --minify

viz-deps:
    cargo depgraph --all-deps --dedup-transitive-deps | dot -Tpng > deps.png
