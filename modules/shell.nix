{ pkgs, ... }:

let
  fluxUtils = pkgs.symlinkJoin {
    name = "flux-utils";
    paths = [
      (pkgs.writeShellApplication {
        name = "killport";
        runtimeInputs = with pkgs; [ lsof findutils procps ];
        text = builtins.readFile ./shell/killport.sh;
      })
      (pkgs.writeShellApplication {
        name = "disk";
        runtimeInputs = with pkgs; [ coreutils gawk ];
        text = builtins.readFile ./shell/disk.sh;
      })
    ];
  };

  yazi = pkgs.yazi.override {
    optionalDeps = with pkgs; [ jq _7zz fd ripgrep fzf zoxide ];
  };
in
{
  programs.zsh = {
    enable = true;
    autosuggestions.enable = true;
    syntaxHighlighting.enable = true;
    histSize = 50000;
    setOptions = [ "HIST_IGNORE_DUPS" "HIST_IGNORE_SPACE" "SHARE_HISTORY" "HIST_FCNTL_LOCK" ];

    shellInit = ''
      zsh-newuser-install() { :; }
    '';

    interactiveShellInit = ''
      unfunction zsh-newuser-install 2>/dev/null
      ${builtins.readFile ./shell/keys.zsh}
      if command -v mise >/dev/null 2>&1; then
          eval "$(mise activate zsh)"
      fi
      ${builtins.readFile ./shell/functions.zsh}
    '';

    promptInit = ''
      format_current_git_branch() {
          local branch
          branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)" || return 0
          [ -n "$branch" ] && printf '(%s)' "$branch"
      }
      setopt PROMPT_SUBST
      PROMPT='%F{magenta}%m %F{cyan}%~%f $(format_current_git_branch)
      %F{green}>%f '
    '';

    shellAliases = {
      ls = "ls --color=auto";
      ll = "ls -lh";
      la = "ls -ah";
      lal = "ls -alh";
      l = "ls";
      c = "clear";
      ax = "chmod +x";
      e = "$EDITOR";
      gs = "git status";
      gp = "git push";
      ga = "git add";
      gd = "git diff";
      gb = "git branch";
      gba = "git branch -a";
      gco = "git checkout";
      ta = "tmux attach";
      tl = "tmux ls -F '#{session_name}'";
      tll = "tmux ls";
      m = "mise";
      y = "yazi";
    };
  };
  users.defaultUserShell = pkgs.zsh;

  environment.interactiveShellInit = ''
    if command -v tiroir >/dev/null 2>&1; then
        eval "$(tiroir export)"
    fi
  '';

  environment.localBinInPath = true;
  environment.variables = {
    GCC_COLORS = "error=01;31:warning=01;35:note=01;36:caret=01;32:locus=01:quote=01";
    MANPAGER = "nvim +Man!";
    RIPGREP_CONFIG_PATH = "/etc/ripgreprc";
    FZF_DEFAULT_COMMAND = "fd --type f --hidden --exclude .git";
  };
  environment.etc.ripgreprc.text = ''
    --smart-case
  '';

  programs.fzf = {
    keybindings = true;
    fuzzyCompletion = true;
  };
  programs.zoxide.enable = true;

  programs.bat = {
    enable = true;
    settings.style = "plain";
  };

  programs.git = {
    enable = true;
    config = {
      init.defaultBranch = "main";
      core.pager = "delta";
      interactive.diffFilter = "delta --color-only";
      merge.conflictstyle = "diff3";
      diff.colorMoved = "default";
      alias.graph = "log --oneline --all --graph --decorate";
    };
  };

  programs.tmux = {
    enable = true;
    terminal = "tmux-256color";
    baseIndex = 1;
    escapeTime = 0;
    historyLimit = 50000;
    aggressiveResize = true;
    clock24 = true;
    plugins = [ pkgs.tmuxPlugins.yank ];
    extraConfig = ''
      set -ag terminal-overrides ",xterm-256color:RGB"
      set -ag terminal-overrides ",xterm-ghostty:RGB"
      set -as terminal-features ",xterm-256color:extkeys"
      set -g extended-keys always
      set -g extended-keys-format csi-u
      set -g allow-passthrough on
      set -g set-clipboard on
      set -g mouse on
      set -g focus-events on
      set -g repeat-time 1000
      set -g display-time 4000
      set -g display-panes-time 800
      set -g renumber-windows on

      bind c new-window -c "#{pane_current_path}"
      bind '"' split-window -c "#{pane_current_path}"
      bind % split-window -h -c "#{pane_current_path}"
      bind | split-window -h -c "#{pane_current_path}"
      bind - split-window -c "#{pane_current_path}"
      bind Tab last-window
      bind h select-pane -L
      bind j select-pane -D
      bind k select-pane -U
      bind l select-pane -R
      bind -r J resize-pane -D 3
      bind -r K resize-pane -U 3
      bind -r H resize-pane -L 3
      bind -r L resize-pane -R 3
      setw -g mode-keys vi
      bind -T copy-mode-vi v send-keys -X begin-selection
      bind -T copy-mode-vi C-v send-keys -X rectangle-toggle
      bind -T copy-mode-vi y send-keys -X copy-selection-and-cancel
      bind x confirm-before -p "Kill pane #P? (y/n)" kill-pane
      bind X confirm-before -p "Kill window #W? (y/n)" kill-window
      bind t display-popup -E -w 80% -h 80% -T " Terminal " -d "#{pane_current_path}"
      bind -n M-1 select-window -t 1
      bind -n M-2 select-window -t 2
      bind -n M-3 select-window -t 3
      bind -n M-4 select-window -t 4
      bind -n M-5 select-window -t 5
      bind -n M-6 select-window -t 6
      bind -n M-7 select-window -t 7
      bind -n M-8 select-window -t 8
      bind -n M-9 select-window -t 9

      set -g status-position top
      set -g status-left "#[fg=colour8] #S "
      set -g status-style "bg=default,fg=default"
      set -g window-status-style "bg=default,fg=colour8"
      set -g window-status-current-style "bg=default,fg=default,bold"
      set -g window-status-format " #W "
      set -g window-status-current-format " #[fg=default,bold]#W "
    '';
  };

  environment.systemPackages = (with pkgs; [
    curl
    ripgrep
    fd
    fzf
    jq
    mise
    bun
    delta
    tree
    lsof
    gh
    yq-go
    shellcheck
    shfmt
  ]) ++ [ yazi fluxUtils ];
}
