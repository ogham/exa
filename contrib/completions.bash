_exa()
{
    cur=${COMP_WORDS[COMP_CWORD]}
    prev=${COMP_WORDS[COMP_CWORD-1]}

    case "$prev" in
        -'?'|--help|-v|--version)
            return
            ;;

        -L|--level)
            COMPREPLY=( $( compgen -W '{0..9}' -- "$cur" ) )
            return
            ;;

        -s|--sort)
            COMPREPLY=( $( compgen -W 'name filename Name Filename size filesize extension Extension modified accessed created none inode --' -- "$cur" ) )
            return
            ;;

        -t|--time)
            COMPREPLY=( $( compgen -W 'accessed modified created --' -- $cur ) )
            return
            ;;
    esac

    case "$cur" in
        -*)
            COMPREPLY=( $( compgen -W '$( _parse_help "$1" )' -- "$cur" ) )
            ;;

        *)
            _filedir
            ;;
    esac
} &&
complete -o filenames -o bashdefault -F _exa exa
