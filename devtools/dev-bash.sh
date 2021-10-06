# This file gets executed when a user starts a `bash` shell, usually because
# they’ve just started a new Vagrant session with `vagrant ssh`. It configures
# some (but not all) of the commands that you can use.


# Display the installed versions of tools.
# help banner
bash /vagrant/devtools/dev-versions.sh


# Configure the Cool Prompt™ (not actually trademarked).
# The Cool Prompt tells you whether you’re in debug or strict mode, whether
# you have colours configured, and whether your last command failed.
nonzero_return() { RETVAL=$?; [ "$RETVAL" -ne 0 ] && echo "$RETVAL "; }
debug_mode()  { [ "$EXA_DEBUG" == "trace" ] && echo -n "trace-"; [ -n "$EXA_DEBUG" ] && echo "debug "; }
strict_mode() { [ -n "$EXA_STRICT" ] && echo "strict "; }
lsc_mode()    { [ -n "$LS_COLORS" ]  && echo "lsc "; }
exac_mode()   { [ -n "$EXA_COLORS" ] && echo "exac "; }
export PS1="\[\e[1;36m\]\h \[\e[32m\]\w \[\e[31m\]\`nonzero_return\`\[\e[35m\]\`debug_mode\`\[\e[32m\]\`lsc_mode\`\[\e[1;32m\]\`exac_mode\`\[\e[33m\]\`strict_mode\`\[\e[36m\]\\$\[\e[0m\] "


# The ‘debug’ function lets you switch debug mode on and off.
# Turn it on if you need to see exa’s debugging logs.
debug() {
  case "$1" in
    ""|"on")  export EXA_DEBUG=1 ;;
    "off")    export EXA_DEBUG= ;;
    "trace")  export EXA_DEBUG=trace ;;
    "status") [ -n "$EXA_DEBUG" ] && echo "debug on" || echo "debug off" ;;
    *)        echo "Usage: debug on|off|trace|status"; return 1 ;;
  esac;
}

# The ‘strict’ function lets you switch strict mode on and off.
# Turn it on if you’d like exa’s command-line arguments checked.
strict() {
  case "$1" in
    "on")  export EXA_STRICT=1 ;;
    "off") export EXA_STRICT= ;;
    "")    [ -n "$EXA_STRICT" ] && echo "strict on" || echo "strict off" ;;
    *)     echo "Usage: strict on|off"; return 1 ;;
  esac;
}

# The ‘colors’ function sets or unsets the ‘LS_COLORS’ and ‘EXA_COLORS’
# environment variables. There’s also a ‘hacker’ theme which turns everything
# green, which is usually used for checking that all colour codes work, and
# for looking cool while you phreak some mainframes or whatever.
colors() {
  case "$1" in
    "ls")
      export LS_COLORS="di=34:ln=35:so=32:pi=33:ex=31:bd=34;46:cd=34;43:su=30;41:sg=30;46:tw=30;42:ow=30;43"
      export EXA_COLORS="" ;;
    "hacker")
      export LS_COLORS="di=32:ex=32:fi=32:pi=32:so=32:bd=32:cd=32:ln=32:or=32:mi=32"
      export EXA_COLORS="ur=32:uw=32:ux=32:ue=32:gr=32:gw=32:gx=32:tr=32:tw=32:tx=32:su=32:sf=32:xa=32:sn=32:sb=32:df=32:ds=32:uu=32:un=32:gu=32:gn=32:lc=32:lm=32:ga=32:gm=32:gd=32:gv=32:gt=32:xx=32:da=32:in=32:bl=32:hd=32:lp=32:cc=32:" ;;
    "off")
      export LS_COLORS=
      export EXA_COLORS= ;;
    "")
      [ -n "$LS_COLORS" ]  && echo "LS_COLORS=$LS_COLORS"   || echo "ls-colors off"
      [ -n "$EXA_COLORS" ] && echo "EXA_COLORS=$EXA_COLORS" || echo "exa-colors off" ;;
    *) echo "Usage: ls-colors ls|hacker|off"; return 1 ;;
  esac;
}
