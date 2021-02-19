# zpack

Heavily influenced by warp-packer but with emphasis on Zstandard as packager.

#Install with:

Step 1):
cargo build --package Zrun --release --target ...

Step 2):
cargo install --path Zpack --target ...

#Or automated one liner with cargo make installed:

cargo make --no-workspace --makefile ./cargo.toml install

Create self-contained single binary application

#USAGE:

zpack --exec <exec> --input_dir <input_dir> --output <output>

#FLAGS:

-h, --help       Prints help information

-V, --version    Prints version information

#OPTIONS:

-i, --input_dir <input_dir>    Sets the input directory containing the application and dependencies

-e, --exec <exec>              Sets the application executable file name

-o, --output <output>          Sets the resulting self-contained application file name
