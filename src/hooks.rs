use std::env;

pub fn generate_zsh_hook() -> String {
    let self_path = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "wherearethey".into());

    format!(r#"# wherearethey — shell hooks for tracking installs
# Add to ~/.zshrc: eval "$(wherearethey hook zsh)"

__wat_bin="{self_path}"

# Wrap brew
brew() {{
    command brew "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            install|reinstall)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log brew install "${{pkgs[@]}}" 2>/dev/null
                ;;
            uninstall|remove|rm)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log brew uninstall "${{pkgs[@]}}" 2>/dev/null
                ;;
        esac
    fi
    return $exit_code
}}

# Wrap npm
npm() {{
    command npm "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            install|i|add)
                if [[ " $* " == *" -g "* ]] || [[ " $* " == *" --global "* ]]; then
                    shift
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log npm install "${{pkgs[@]}}" 2>/dev/null
                fi
                ;;
            uninstall|un|remove|rm)
                if [[ " $* " == *" -g "* ]] || [[ " $* " == *" --global "* ]]; then
                    shift
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log npm uninstall "${{pkgs[@]}}" 2>/dev/null
                fi
                ;;
        esac
    fi
    return $exit_code
}}

# Wrap pnpm
pnpm() {{
    command pnpm "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            add)
                if [[ " $* " == *" -g "* ]] || [[ " $* " == *" --global "* ]]; then
                    shift
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log pnpm install "${{pkgs[@]}}" 2>/dev/null
                fi
                ;;
            remove|rm)
                if [[ " $* " == *" -g "* ]] || [[ " $* " == *" --global "* ]]; then
                    shift
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log pnpm uninstall "${{pkgs[@]}}" 2>/dev/null
                fi
                ;;
        esac
    fi
    return $exit_code
}}

# Wrap bun
bun() {{
    command bun "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            install|add)
                if [[ " $* " == *" -g "* ]] || [[ " $* " == *" --global "* ]]; then
                    shift
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log bun install "${{pkgs[@]}}" 2>/dev/null
                fi
                ;;
            remove|rm)
                if [[ " $* " == *" -g "* ]] || [[ " $* " == *" --global "* ]]; then
                    shift
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log bun uninstall "${{pkgs[@]}}" 2>/dev/null
                fi
                ;;
        esac
    fi
    return $exit_code
}}

# Wrap cargo
cargo() {{
    command cargo "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            install|binstall)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log cargo install "${{pkgs[@]}}" 2>/dev/null
                ;;
            uninstall)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log cargo uninstall "${{pkgs[@]}}" 2>/dev/null
                ;;
        esac
    fi
    return $exit_code
}}

# Wrap go
go() {{
    command go "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 && "$1" == "install" ]]; then
        shift
        local pkgs=()
        for arg in "$@"; do
            [[ "$arg" != -* ]] && pkgs+=("$arg")
        done
        [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log go install "${{pkgs[@]}}" 2>/dev/null
    fi
    return $exit_code
}}

# Wrap pip3 / pip
pip3() {{
    command pip3 "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            install)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log pip install "${{pkgs[@]}}" 2>/dev/null
                ;;
            uninstall)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log pip uninstall "${{pkgs[@]}}" 2>/dev/null
                ;;
        esac
    fi
    return $exit_code
}}
pip() {{ pip3 "$@"; }}

# Wrap pipx
pipx() {{
    command pipx "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            install|inject)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log pipx install "${{pkgs[@]}}" 2>/dev/null
                ;;
            uninstall)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log pipx uninstall "${{pkgs[@]}}" 2>/dev/null
                ;;
        esac
    fi
    return $exit_code
}}

# Wrap uv
uv() {{
    command uv "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        if [[ "$1" == "tool" ]]; then
            case "$2" in
                install)
                    shift 2
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log uv install "${{pkgs[@]}}" 2>/dev/null
                    ;;
                uninstall)
                    shift 2
                    local pkgs=()
                    for arg in "$@"; do
                        [[ "$arg" != -* ]] && pkgs+=("$arg")
                    done
                    [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log uv uninstall "${{pkgs[@]}}" 2>/dev/null
                    ;;
            esac
        fi
    fi
    return $exit_code
}}

# Wrap gem
gem() {{
    command gem "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        case "$1" in
            install)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log gem install "${{pkgs[@]}}" 2>/dev/null
                ;;
            uninstall)
                shift
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log gem uninstall "${{pkgs[@]}}" 2>/dev/null
                ;;
        esac
    fi
    return $exit_code
}}

# Wrap deno
deno() {{
    command deno "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 && "$1" == "install" ]]; then
        shift
        local pkgs=()
        for arg in "$@"; do
            [[ "$arg" != -* ]] && pkgs+=("$arg")
        done
        [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log deno install "${{pkgs[@]}}" 2>/dev/null
    fi
    return $exit_code
}}

# Wrap composer
composer() {{
    command composer "$@"
    local exit_code=$?
    if [[ $exit_code -eq 0 && "$1" == "global" ]]; then
        case "$2" in
            require)
                shift 2
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log composer install "${{pkgs[@]}}" 2>/dev/null
                ;;
            remove)
                shift 2
                local pkgs=()
                for arg in "$@"; do
                    [[ "$arg" != -* ]] && pkgs+=("$arg")
                done
                [[ ${{#pkgs[@]}} -gt 0 ]] && "$__wat_bin" log composer uninstall "${{pkgs[@]}}" 2>/dev/null
                ;;
        esac
    fi
    return $exit_code
}}
"#)
}
