{
  description = "Home Dashboard";
  nixConfig.bash-prompt = "\[nix-dev-HD\]$ ";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    #nixpkgs.follows = "rust-overlay/nixpkgs";
    nixpkgs.url = "nixpkgs/nixos-22.11";
  };

  outputs = { self, nixpkgs, rust-overlay }:

    let pkgs = nixpkgs.legacyPackages.x86_64-linux.extend (import rust-overlay);

    in  {
      packages.x86_64-linux.home-dashboard = pkgs.rustPlatform.buildRustPackage {
          pname = "home-dashboard";
          version = "0.1";
          src = self;

          nativeBuildInputs = with pkgs; [
             rust-bin.stable.latest.minimal
             dbus
             pkg-config
          ];

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          installFlags = [ "PREFIX=$(out)" ];

          buildInputs = with pkgs; [ gtk3 plan9port ];
    };

    packages.x86_64-linux.default = self.packages.x86_64-linux.home-dashboard;
  };
}
