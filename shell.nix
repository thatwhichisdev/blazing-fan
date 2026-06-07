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
    sha256 = "sha256-eeq8tmjoth7VJGVp65a5e+dEQFk/lzKuSq6jWETXwb4=";
  };
in
pkgs.mkShellNoCC {
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

  STARSHIP_CONFIG = "./.starship/starship.toml";

  shellHook = ''
    exec nu --config ./.nu/config.nu
  '';
}
