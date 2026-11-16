{ pkgs, ... }:

let
  vim = pkgs.symlinkJoin {
    name = "vim-${pkgs.vim.version}";
    paths = [ pkgs.vim ];
    nativeBuildInputs = [ pkgs.makeWrapper ];
    postBuild = ''
      wrapProgram $out/bin/vim --add-flags "--cmd 'source ${./editors/vimrc}'"
    '';
    meta.mainProgram = "vim";
  };

  grammars = p: with p; [
    bash
    css
    diff
    go
    html
    javascript
    json
    nix
    python
    rust
    toml
    tsx
    typescript
    yaml
  ];
in
{
  programs.neovim = {
    enable = true;
    defaultEditor = true;
    configure = {
      customLuaRC = ''
        local user = vim.fn.stdpath("config")
        local lua_rc, vim_rc = user .. "/init.lua", user .. "/init.vim"
        if vim.uv.fs_stat(lua_rc) then
          vim.env.MYVIMRC = lua_rc
          dofile(lua_rc)
        elseif vim.uv.fs_stat(vim_rc) then
          vim.env.MYVIMRC = vim_rc
          vim.cmd.source(vim_rc)
        else
          dofile("${./editors/nvim.lua}")
        end
      '';
      packages.flux.start = with pkgs.vimPlugins; [
        plenary-nvim
        telescope-nvim
        (nvim-treesitter.withPlugins grammars)
        gitsigns-nvim
        which-key-nvim
        mini-nvim
        nvim-lspconfig
      ];
    };
  };

  programs.vim = {
    enable = true;
    package = vim;
  };

  environment.etc."vis/visrc.lua".source = ./editors/visrc.lua;

  environment.systemPackages = with pkgs; [
    vis
    stylua
    lua-language-server
  ];
}
