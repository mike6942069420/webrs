{
  description = "dev shell for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = { self, nixpkgs }: 
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        name = "dev-shell";

        buildInputs = with pkgs; [
          # random ci/cd tools
          bashInteractive
          coreutils
          gnumake
          git
          docker

          # rust toolchain
          rustup

          # test
          apacheHttpd


        ];

        # Setup environment to use musl target by default
        shellHook = ''
          rustup install stable
          rustup default stable
          rustup target add x86_64-unknown-linux-musl
        '';
      };

      devShell.${system} = self.devShells.${system}.default;
    };
}
