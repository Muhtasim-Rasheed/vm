base="${1%.*}"
assembly="$base.asm"
binary="$base.bin"
cargo run --bin kilnc -- $1 $assembly
echo
echo "compiled"
echo
cargo run --bin assembler -- $assembly $binary
echo
echo "assembled"
echo
if [[ "$2" == "--debug" ]]; then
		cargo run --bin vm -- $binary --debug
else
		cargo run --bin vm -- $binary
fi
