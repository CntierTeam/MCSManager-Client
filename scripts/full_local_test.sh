#!/usr/bin/env bash
# Full local MCSManager integration test for mcsm CLI.
set -u
export PATH="${HOME}/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:${PATH}"

ROOT=/projectsDir/IdeaProjects/MCSManager-CLI
MCSM="$ROOT/target/debug/mcsm"
URL=http://127.0.0.1:33333
CFG=/tmp/mcsm-test-config.toml
REPORT=/tmp/mcsm-full-test-report2.txt
: > "$REPORT"

pass=0; fail=0; skip=0

log() { echo "$*" | tee -a "$REPORT"; }
try() {
  local name="$1"; shift
  log "=== $name ==="
  log "+ $*"
  if "$@" >>"$REPORT" 2>&1; then
    log "PASS $name"
    pass=$((pass+1))
  else
    log "FAIL $name"
    fail=$((fail+1))
  fi
  log ""
}
skip() {
  log "=== $1 === SKIP ($2)"
  skip=$((skip+1))
  log ""
}

cd "$ROOT"
cargo build -p mcsm >>"$REPORT" 2>&1

# Ensure services
if ! curl -fsS "$URL/api/auth/status" >/dev/null; then
  log "ERROR: panel not reachable at $URL"
  exit 1
fi

# Login + API key
TOKEN=$(curl -sS -c /tmp/mcsm-cookies.txt -X POST "$URL/api/auth/login" \
  -H 'Content-Type: application/json' -H 'X-Requested-With: XMLHttpRequest' \
  -d '{"username":"admin","password":"AdminTest123!"}' | python3 -c 'import sys,json; print(json.load(sys.stdin)["data"])')
APIKEY=$(curl -sS -b /tmp/mcsm-cookies.txt -X PUT "$URL/api/auth/api?token=$TOKEN" \
  -H 'Content-Type: application/json' -H 'X-Requested-With: XMLHttpRequest' \
  -d '{"enable":true}' | python3 -c 'import sys,json; print(json.load(sys.stdin)["data"])')

DAEMON=$(curl -sS "$URL/api/service/remote_services_list?apikey=$APIKEY" \
  -H 'X-Requested-With: XMLHttpRequest' | python3 -c 'import sys,json; print(json.load(sys.stdin)["data"][0]["uuid"])')

cat > "$CFG" <<EOF
panel_url = "$URL"
api_key = "$APIKEY"
default_daemon_id = "$DAEMON"
EOF

log "URL=$URL DAEMON=$DAEMON APIKEY=***"

try config-show "$MCSM" --config "$CFG" config show --json
try auth-status "$MCSM" --config "$CFG" auth status --json
try auth-whoami "$MCSM" --config "$CFG" auth whoami --json
try overview "$MCSM" --config "$CFG" overview --json
try node-list "$MCSM" --config "$CFG" node list --json
try node-system "$MCSM" --config "$CFG" node system --json
try settings-get "$MCSM" --config "$CFG" settings get --json
try user-list "$MCSM" --config "$CFG" user list --json
try audit-recent "$MCSM" --config "$CFG" audit recent --json
try market-list "$MCSM" --config "$CFG" market list --json
try instance-list-global "$MCSM" --config "$CFG" instance list --global --json

CREATE='{"nickname":"cli-full","startCommand":"sleep 120","stopCommand":"^C","cwd":"","ie":"utf-8","oe":"utf-8","createDatetime":"","lastDatetime":"","type":"universal","tag":[],"maxSpace":null,"endTime":0,"docker":{},"enableRcon":false,"rconPassword":"","rconPort":0,"rconIp":"","terminalOption":{"haveColor":false,"pty":false},"eventTask":{"autoStart":false,"autoRestart":false,"ignore":false},"pingConfig":{},"processType":"general","fileCode":"utf-8","updateCommand":"","crlf":1}'
try instance-create "$MCSM" --config "$CFG" instance create --json-body "$CREATE" --json

UUID=$("$MCSM" --config "$CFG" instance list --json | python3 -c '
import sys,json
p=json.load(sys.stdin)
for i in p.get("data") or []:
  if (i.get("config") or {}).get("nickname")=="cli-full":
    print(i["instanceUuid"]); break
')
log "UUID=$UUID"

try instance-get "$MCSM" --config "$CFG" instance get "$UUID" --json
try instance-open "$MCSM" --config "$CFG" instance open "$UUID" --json
sleep 1
try instance-cmd "$MCSM" --config "$CFG" instance cmd "$UUID" "noop" --json
try terminal-log "$MCSM" --config "$CFG" terminal log "$UUID" --json
try instance-stop "$MCSM" --config "$CFG" instance stop "$UUID" --json
sleep 1
try instance-kill "$MCSM" --config "$CFG" instance kill "$UUID" --json

try file-ls "$MCSM" --config "$CFG" file ls "$UUID" --target / --json
try file-mkdir "$MCSM" --config "$CFG" file mkdir "$UUID" cli-dir --json
try file-touch "$MCSM" --config "$CFG" file touch "$UUID" cli-dir/a.txt --json

# Multipart upload broken in MCSManager 10.18.1 webpack+formidable packaging
skip file-upload "MCSM 10.18.1 daemon webpack formidable plugins MODULE_NOT_FOUND"

# Write via file edit API (covered by panel), then download via passport
try file-edit-api curl -sS -f -X PUT "$URL/api/files?daemonId=$DAEMON&uuid=$UUID&apikey=$APIKEY" \
  -H 'Content-Type: application/json' -H 'X-Requested-With: XMLHttpRequest' \
  -d '{"target":"cli-dir/a.txt","text":"content-from-test"}'
try file-download "$MCSM" --config "$CFG" file download "$UUID" cli-dir/a.txt /tmp/mcsm-dl-final.txt --json
if [[ "$(cat /tmp/mcsm-dl-final.txt 2>/dev/null)" == "content-from-test" ]]; then
  log "PASS file-download-verify"; pass=$((pass+1))
else
  log "FAIL file-download-verify content='$(cat /tmp/mcsm-dl-final.txt 2>/dev/null)'"; fail=$((fail+1))
fi
log ""
try file-rm "$MCSM" --config "$CFG" file rm "$UUID" cli-dir/a.txt --json

try schedule-list "$MCSM" --config "$CFG" schedule list "$UUID" --json
try schedule-add "$MCSM" --config "$CFG" schedule add "$UUID" --name cli-sched --time 5 --action command --payload list --type-code 1 --count 1 --json
try schedule-rm "$MCSM" --config "$CFG" schedule rm "$UUID" cli-sched --json
try java-list "$MCSM" --config "$CFG" java list "$UUID" --json
try config-list "$MCSM" --config "$CFG" instance config-list "$UUID" --json
try mod-list "$MCSM" --config "$CFG" mod list "$UUID" --json
try audit-search "$MCSM" --config "$CFG" audit search --keyword admin --json

# unique user name
UNAME="testuser$(date +%s)"
try user-create "$MCSM" --config "$CFG" user create "$UNAME" TestUser123! --permission 1 --json

skip env-images "no docker.sock on host"
skip env-containers "no docker.sock on host"

try stream-channel curl -sS -f -X POST "$URL/api/protected_instance/stream_channel?daemonId=$DAEMON&uuid=$UUID&apikey=$APIKEY" -H 'X-Requested-With: XMLHttpRequest'

try instance-rm "$MCSM" --config "$CFG" instance rm "$UUID" --delete-file --json

log "======== SUMMARY pass=$pass fail=$fail skip=$skip ========"
echo "pass=$pass fail=$fail skip=$skip"
exit $([[ "$fail" -eq 0 ]] && echo 0 || echo 1)
