let
  nixpkgs = builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz";
  };

  fenixSrc = builtins.fetchTarball {
    url = "https://github.com/nix-community/fenix/archive/main.tar.gz";
  };

  pkgs = import nixpkgs {
    overlays = [
      (import "${fenixSrc}/overlay.nix")
    ];
  };

  rustToolchain = pkgs.fenix.fromToolchainFile {
    file = ./rust-toolchain.toml;
    sha256 = "sha256-XSTXenuh9nBL9paMVu/SDiYppNwfKh8Y2BUAngM3xaE=";
  };
in
pkgs.mkShell {
  packages = with pkgs; [
    gitui
    rustToolchain
    laze
    probe-rs-tools
    espup
    espflash
    nushell
    starship
    nerd-fonts.jetbrains-mono
  ];

  STARSHIP_CONFIG = "${./.starship/starship.toml}";

  shellHook = ''
    unset NIX_ENFORCE_PURITY
    exec nu --config ${./.nu/config.nu}
  '';
}
