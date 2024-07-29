nix-build:## 	nix-build
	@type -P nix && nix-build || echo "nix commands not found!"
