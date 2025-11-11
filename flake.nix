# src: https://tonyfinn.com/blog/nix-from-first-principles-flake-edition/nix-8-flakes-and-developer-environments
{
	inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

	outputs = { self, nixpkgs }:
	let
		system = "x86_64-linux";
		pkgs = import nixpkgs {
			inherit system;
			config.allowUnfree = true;
		};
	in {
		devShells.${system}.default = pkgs.mkShell rec {
			packages = with pkgs; [
				libxkbcommon # for minifb
				vulkan-loader # for wgpu
			];
			# Environment variables:
			# RUST_BACKTRACE = "full";
			LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
			WGPU_BACKEND = "vulkan"; # vulkan, metal, dx12, gl
		};
	};
}
