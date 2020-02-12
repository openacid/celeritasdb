fmt:
	# nmp install -g prettier
	prettier --write --print-width 80 --prose-wrap preserve **/*.md
	rustfmt --edition 2018 **/*.rs
