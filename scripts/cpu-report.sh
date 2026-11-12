#!/usr/bin/env bash
# What keeps shoji-bar-4 busy.
#
#   scripts/cpu-report.sh [--sudo] [SECONDS]                 今すぐ SECONDS 秒（既定 10）調べて
#                                                            報告を書く
#   scripts/cpu-report.sh [--sudo] watch [PERCENT] [EVERY]   常駐して、CPU が PERCENT %（既定 8）
#                                                            を 2 回続けて超えたら自動で報告を
#                                                            書く。EVERY 秒（既定 30）ごとに見る
#
# 報告は $XDG_STATE_HOME/shoji-bar-4/reports/cpu-<日時>.txt に書き、要約を表示する。
# perf があればどの関数で時間を使っているかも取る（release ビルドは関数名を残している）。
# 動いているプロセスに perf を付けるのは、kernel.perf_event_paranoid が 2 以上だと root でしか
# できない。--sudo を付けると perf だけを sudo で動かす（watch では始めに一度パスワードを聞き、
# 期限が切れないように更新し続ける）。

set -u

reports="${XDG_STATE_HOME:-$HOME/.local/state}/shoji-bar-4/reports"
here="$(cd "$(dirname "$0")/.." && pwd)"
bin="$here/target/release/shoji-bar-4"
[ -x "$bin" ] || bin="$(command -v shoji-bar-4 || true)"
hz="$(getconf CLK_TCK)"
use_sudo=0

shell_pid() {
    # The oldest one is the shell; `shoji-bar-4 msg …` come and go.
    pgrep -xo shoji-bar-4
}

# utime+stime of the whole process, in ticks.
process_ticks() {
    awk '{ sub(/^.*\) /, ""); print $12 + $13 }' "/proc/$1/stat" 2>/dev/null
}

# One line per thread: tid ticks voluntary nonvoluntary name
threads() {
    local pid=$1 t
    for t in /proc/"$pid"/task/*; do
        local tid=${t##*/}
        local ticks name vol non
        ticks=$(awk '{ sub(/^.*\) /, ""); print $12 + $13 }' "$t/stat" 2>/dev/null) || continue
        name=$(tr ' ' '_' <"$t/comm" 2>/dev/null)
        vol=$(awk '/^voluntary_ctxt_switches/ { print $2 }' "$t/status" 2>/dev/null)
        non=$(awk '/^nonvoluntary_ctxt_switches/ { print $2 }' "$t/status" 2>/dev/null)
        echo "$tid ${ticks:-0} ${vol:-0} ${non:-0} ${name:-?}"
    done
}

# The threads there both before and after: tid, then four fields from before and four from after.
joined() {
    LC_ALL=C join <(LC_ALL=C sort "$1/before") <(LC_ALL=C sort "$1/after")
}

cpu_percent_over() {
    local pid=$1 secs=$2 a b
    a=$(process_ticks "$pid") || return 1
    sleep "$secs"
    b=$(process_ticks "$pid") || return 1
    awk -v a="$a" -v b="$b" -v hz="$hz" -v s="$secs" 'BEGIN { printf "%.1f", (b - a) * 100 / hz / s }'
}

report() {
    local secs=${1:-10}
    local pid
    pid=$(shell_pid) || { echo "shoji-bar-4 が動いていません" >&2; return 1; }
    mkdir -p "$reports"
    local out
    out="$reports/cpu-$(date +%Y%m%d-%H%M%S).txt"
    local tmp
    tmp=$(mktemp -d)

    echo "shoji-bar-4 (pid $pid) を ${secs} 秒調べています…" >&2

    # The shell's own view, and how quickly its UI thread answers.
    local t0 t1 status
    t0=$(date +%s%N)
    status=$(timeout 3 "$bin" msg status 2>&1) || status="(答えなし: $status)"
    t1=$(date +%s%N)
    local answer_ms=$(((t1 - t0) / 1000000))

    # Everything below runs over the same SECONDS.
    threads "$pid" >"$tmp/before"
    local ticks_before
    ticks_before=$(process_ticks "$pid")

    local perf_ok=0
    local perf=(perf)
    [ "$use_sudo" = 1 ] && perf=(sudo -n perf)
    if command -v perf >/dev/null; then
        "${perf[@]}" record -q -e cpu-clock -F 499 --call-graph dwarf,8192 -p "$pid" \
            -o "$tmp/perf.data" -- sleep "$secs" 2>"$tmp/perf.err" &
        local perf_job=$!
    fi
    local monitors=()
    if command -v dbus-monitor >/dev/null; then
        timeout "$secs" dbus-monitor --session --profile >"$tmp/dbus-session" 2>/dev/null &
        monitors+=($!)
        timeout "$secs" dbus-monitor --system --profile >"$tmp/dbus-system" 2>/dev/null &
        monitors+=($!)
    fi
    sleep "$secs"
    threads "$pid" >"$tmp/after"
    local ticks_after
    ticks_after=$(process_ticks "$pid")
    if [ -n "${perf_job:-}" ]; then
        wait "$perf_job" && [ -s "$tmp/perf.data" ] && perf_ok=1
        [ "$use_sudo" = 1 ] && sudo -n chown "$USER" "$tmp/perf.data" 2>/dev/null
    fi
    # Only these: a bare wait would wait for the loop that keeps sudo fresh, which never ends.
    [ "${#monitors[@]}" -gt 0 ] && wait "${monitors[@]}"

    local cpu
    cpu=$(awk -v a="$ticks_before" -v b="$ticks_after" -v hz="$hz" -v s="$secs" \
        'BEGIN { printf "%.1f", (b - a) * 100 / hz / s }')

    {
        echo "# shoji-bar-4 CPU report $(date '+%F %T')"
        echo
        echo "pid $pid, 起動から $(ps -o etime= -p "$pid" | tr -d ' ')、調べた時間 ${secs} 秒"
        echo "shoji-bar-4 $(git -C "$here" rev-parse --short HEAD 2>/dev/null)$(git -C "$here" diff --quiet 2>/dev/null || echo ' (変更あり)')"
        echo "AntiWebUI   $(grep -m1 -o 'AntiWebUI.git?branch=main#[0-9a-f]\{7\}' "$here/Cargo.lock" 2>/dev/null | sed 's/.*#/main /')"
        echo "$(uname -sr)"
        echo
        echo "## 全体"
        echo
        echo "CPU ${cpu}%（1 コアを 100% として）"
        grep -E '^(VmRSS|RssAnon|Threads)' "/proc/$pid/status" | tr -s ' \t' ' '
        echo
        echo "## シェルの状態（msg status、答えるまで ${answer_ms} ms）"
        echo
        echo "$status"
        echo
        echo "ticking はフレームを求め続けている時計の数。0 のはずのときに 1 以上なら、止まらない"
        echo "アニメーションがある。答えるまでが 100 ms を超えるなら、UI スレッドが詰まっている。"
        echo
        echo "## スレッドごと"
        echo
        echo "目覚め/秒 は自分から眠った回数。UI スレッド（tid = pid）で 60〜120 前後なら、"
        echo "フレームを描き続けている。"
        echo
        printf '%-8s %-18s %7s %11s %11s\n' tid 名前 CPU% 目覚め/秒 横取り/秒
        joined "$tmp" | awk -v hz="$hz" -v s="$secs" -v pid="$pid" '
            {
                cpu = ($6 - $2) * 100 / hz / s
                wake = ($7 - $3) / s
                pre = ($8 - $4) / s
                name = $5 (($1 == pid) ? "[UI]" : "")
                printf "%-8s %-18s %7.1f %11.1f %11.1f\n", $1, name, cpu, wake, pre
            }' | sort -k3,3 -rn | head -25
        echo

        if [ "$perf_ok" = 1 ]; then
            echo "## 時間を使っている関数（perf、自分の時間）"
            echo
            perf report -i "$tmp/perf.data" --stdio --no-children -g none \
                --sort comm,dso,symbol --percent-limit 0.7 2>/dev/null |
                grep -v '^#' | grep -v '^$' | head -60
            echo
            echo "## 呼び出し元を含めた時間（perf、上位）"
            echo
            perf report -i "$tmp/perf.data" --stdio --children -g none \
                --sort symbol --percent-limit 4 2>/dev/null |
                grep -v '^#' | grep -v '^$' | head -60
            echo
        elif command -v perf >/dev/null; then
            echo "## perf"
            echo
            echo "取れませんでした:"
            head -3 "$tmp/perf.err"
            [ "$use_sudo" = 1 ] || echo "→ --sudo を付けて動かすと取れます。"
            echo
        else
            echo "## perf"
            echo
            echo "perf がないので関数ごとの時間は取っていません（pacman -S perf）。"
            echo
        fi

        for bus in session system; do
            [ -s "$tmp/dbus-$bus" ] || continue
            echo "## D-Bus（$bus）のシグナル、${secs} 秒で多いもの"
            echo
            awk '$1 == "sig" { n[$3 " " $(NF-1) "." $NF]++; total++ }
                END { for (k in n) printf "%6d  %s\n", n[k], k; printf "%6d  合計\n", total }' \
                "$tmp/dbus-$bus" | sort -rn | head -15
            echo
        done

        local gpu
        if command -v nvidia-smi >/dev/null &&
            gpu=$(timeout 5 nvidia-smi pmon -c 2 -s u 2>/dev/null); then
            echo "## GPU（nvidia-smi pmon）"
            echo
            awk -v pid="$pid" 'NR <= 2 || $2 == pid' <<<"$gpu"
            echo
        fi

        echo "## 見立て"
        echo
        local ui_wake ticking
        ui_wake=$(joined "$tmp" |
            awk -v s="$secs" -v pid="$pid" '$1 == pid { printf "%.0f", ($7 - $3) / s }')
        ticking=$(sed -n 's/.*ticking=\([0-9]*\).*/\1/p' <<<"$status")
        if [ "${ui_wake:-0}" -ge 40 ] && [ "${ticking:-0}" -gt 0 ]; then
            echo "- UI スレッドが毎秒 ${ui_wake} 回動いていて、時計が ${ticking} 個動いている。"
            echo "  ドックの起動中の跳ね（4 秒で止まる）か、再生中のイコライザー（毎秒 30 回）のはず。"
            echo "  どちらでもないなら clock_while / clock_at の止まり方を疑う。"
        elif [ "${ui_wake:-0}" -ge 40 ]; then
            echo "- UI スレッドが毎秒 ${ui_wake} 回動いているが、シェルの時計は動いていない。"
            echo "  ばね（follow / animated）やトランジションが止まらないか、Wayland や D-Bus からの"
            echo "  イベントが絶えず来ている。上の perf と D-Bus の表を見る。"
        else
            echo "- UI スレッドはフレームを描き続けてはいない（毎秒 ${ui_wake:-?} 回）。"
            echo "  CPU を使っているのが UI 以外のスレッドなら、その名前と perf の dso を見る。"
        fi
    } >"$out"
    rm -rf "$tmp"

    sed -n '/^## 全体/,/^## スレッドごと/p' "$out" | head -12
    echo "…"
    sed -n '/^## 見立て/,$p' "$out"
    echo
    echo "報告: $out"
}

watch() {
    local limit=${1:-8} every=${2:-30} over=0
    echo "shoji-bar-4 の CPU を ${every} 秒ごとに見ています（${limit}% を 2 回続けて超えたら報告）。Ctrl+C で終わり。" >&2
    while :; do
        local pid cpu
        if ! pid=$(shell_pid); then
            sleep "$every"
            continue
        fi
        cpu=$(cpu_percent_over "$pid" 5) || { sleep "$every"; continue; }
        if awk -v c="$cpu" -v l="$limit" 'BEGIN { exit !(c > l) }'; then
            over=$((over + 1))
            echo "$(date +%T) CPU ${cpu}%" >&2
        else
            over=0
        fi
        if [ "$over" -ge 2 ]; then
            report 10
            if command -v notify-send >/dev/null; then
                notify-send -a shoji-bar-4 "shoji-bar-4 の CPU が ${cpu}%" "報告を $reports に書きました"
            fi
            over=0
            # Once is enough for one episode.
            sleep 600
        fi
        sleep "$every"
    done
}

if [ "${1:-}" = --sudo ]; then
    use_sudo=1
    shift
    sudo -v || exit 1
    # Keeps the password fresh while this runs, for perf in a report much later.
    while sleep 60; do sudo -n -v 2>/dev/null || exit; done &
    fresh=$!
    trap 'kill "$fresh" 2>/dev/null' EXIT
fi

case "${1:-}" in
    watch) shift; watch "$@" ;;
    -h | --help) sed -n '2,15p' "$0" ;;
    *) report "${1:-10}" ;;
esac
