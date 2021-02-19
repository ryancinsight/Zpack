# zpack

Heavily influenced by warp-packer but with emphasis on Zstandard as packager.

Install with:
cargo build --package Zrun --release --target ...
cargo install --path Zpack --target ...

Or automated one liner with cargo make installed:
cargo make --no-workspace --makefile ./cargo.toml install