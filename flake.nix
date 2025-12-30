{
  description = "Development Nix flake for Nori Codex CLI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems f;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          codex-rs = pkgs.callPackage ./codex-rs { };
        in
        {
          codex-rs = codex-rs;
          default = codex-rs;
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            # Pull in dependencies from your package definition
            inputsFrom = [ (pkgs.callPackage ./codex-rs { }) ];

            # Define the library path so the binary finds Nix's OpenSSL at runtime
            shellHook = ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}:$LD_LIBRARY_PATH
            '';
          };
        }
      );
    };
}
