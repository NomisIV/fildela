{
  description = "fildela - http file sharing service";

  inputs = {
    nixpkgs.url = github:nixos/nixpkgs;
  };

  outputs = inputs:
    with inputs;
    let
      systems = [
        "aarch64-linux"
        "aarch64-darwin"
        "i686-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      config = system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        defaultPackage.${system} = pkgs.rustPlatform.buildRustPackage {
          pname = "fildela";
          version = "0.1.0";
          src = self;
          cargoSha256 = "sha256-O+8WZHp4HjHI5LbAvr//OVbK2ru+Of1p8EwJ/9fn7zc=";
        };

        nixosModule = import ./module.nix;

        devShell.${system} = pkgs.mkShell {
          buildInputs = with pkgs; [ rustc cargo rustfmt ];
        };
      };
    in builtins.foldl' (acc: system: acc // (config system)) { } systems;
  }
