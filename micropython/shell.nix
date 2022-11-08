with import <nixpkgs> {};

let
    rpi_ws281x = with python3Packages; buildPythonPackage rec {
      name = "rpi_ws281x";
      version = "v0.1";

      src = pkgs.fetchFromGitHub {
      	 owner = "jgarff";
	 repo = "${name}";
         url = "https://github.com/jgarff/rpi_ws281x.git";
         rev = "ee7522e3b053950af33bc7e4364742cd3aeaf594";
	 sha256 = "0jdbg7s68zrx475qdlpffjh045dzmlbvbpz4ikd8yxaa20hyzb4k";
      };

      propagatedBuildInputs = [ numpy toolz setproctitle ];

      meta = {
        homepage = "https://github.com/jgarff/rpi_ws281x";
        description = "Userspace led driver";
        maintainers = with maintainers; [ jgarff ];
      };
    };

in mkShell {
  name = "rpi_ws281x";
  buildInputs = [ rpi_ws281x ];
  shellHook = ''
  '';
}
