{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ inputs.fenix.overlays.default ];
        };
      in
      with pkgs;
      {
        formatter = nixfmt-tree;
        inherit inputs;
        devShells.default = mkShell {
          buildInputs = [
            (fenix.combine [
              fenix.stable.defaultToolchain
              fenix.stable.rust-src
            ])
          ];
        };
      }
    );
}
