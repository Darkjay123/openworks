export PATH=/workspace/bin:$HOME/.cargo/bin:$PATH
NET=testnet
source "$(dirname "$0")/../deploy.env"
LOG="$(dirname "$0")/../testnet-log.md"
addr(){ stellar keys address $1; }
# inv <label> <source> <contract> -- args...
inv(){ local label="$1" src="$2" id="$3"; shift 3
  local out; out=$(stellar contract invoke --id "$id" --source "$src" --network $NET -- "$@" 2>&1); local rc=$?
  local tx; tx=$(echo "$out" | grep -o 'https://stellar.expert/explorer/testnet/tx/[a-f0-9]*' | tail -1)
  local res; res=$(echo "$out" | grep -v -e '🔗' -e '✅' -e 'ℹ️' -e '^$' -e 'Signing' -e 'Simulat' -e 'Submitting' -e 'Event' -e '📅' -e '🌎' | tail -1)
  if [ $rc -eq 0 ]; then echo "| $label | ok | ${res:-} | ${tx:-} |" | tee -a "$LOG"; else echo "| $label | REJECTED | $(echo "$out" | grep -o 'Error(Contract, #[0-9]*)' | head -1) | |" | tee -a "$LOG"; fi
  LAST="$res"; return 0; }
view(){ stellar contract invoke --id "$1" --source ow-funder --network $NET --send=no -- "${@:2}" 2>/dev/null; }
