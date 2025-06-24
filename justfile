set windows-powershell := true

pull: prettier fix test


test:
	cargo test --workspace

fix: fmt
	cargo clippy --workspace --fix --allow-dirty --allow-staged

fmt:
	cargo fmt --all

prettier:
	# install on Debian: sudo snap install node --classic
	# npx is the command to run npm package, node is the runtime
	npx prettier -w .
