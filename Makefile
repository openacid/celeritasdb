fmt:
	# nmp install -g prettier
	prettier --write --print-width 80 --prose-wrap preserve **/*.md
	find . -name "*.rs" -exec rustfmt --edition 2018 {} ';'

# build protobuf files into *.rs
pb:
	( cd pbbuild && cargo run; )
