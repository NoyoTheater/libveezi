{
  description = "mfc";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs { inherit system; };
          in
          f { inherit system pkgs; }
        );
    in
    {
      devShells = forAllSystems (
        { pkgs, system }: {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              fenix.packages.${system}.complete.toolchain
              nixfmt
            ];
          };
        }
      );
    };
}
