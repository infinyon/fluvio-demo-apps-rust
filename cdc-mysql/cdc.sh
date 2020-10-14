#!/usr/bin/env bash

set -x

readonly PROGNAME=$(basename "${0}")
readonly PROGDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
readonly ARGS=("$@")

[[ -x "$(command -v "mysql")" ]] || { echo "Missing mysql"; exit; }
[[ -x "$(command -v "docker")" ]] || { echo "Missing docker"; exit; }
[[ -x "$(command -v "fluvio")" ]] || { echo "Missing fluvio"; exit; }

pkill_producer_consumer() {
  pkill cdc-producer
  pkill cdc-consumer
}

fluvio_uninstall() {
  fluvio cluster uninstall --local
}

docker_kill_mysql() {
  docker kill mysql-consumer mysql-producer
}

docker_rm_mysql() {
  docker rm mysql-consumer mysql-producer
}

docker_rmi_mysql() {
  docker rmi mysql-80
}

clear_mysql_volume() {
  # Use first arg for mysql volume mount, or ~/mysql-cdc as default
  local path="${1:-"${HOME}/mysql-cdc"}"

  sudo rm -rf "${path}"
}

fluvio_install() {
  fluvio cluster install --local
}

# shellcheck disable=SC2120
fluvio_topic() {
  local topic="${1:-"rust-mysql-cdc"}"; shift
  fluvio topic create "${topic}"
  sleep 1
}

docker_mysql() {
  local name=${1:-"mysql-producer"}; shift
  local path=${1:-"${HOME}/mysql-cdc/cdc-producer"}; shift
  local port=${1:-"3080"}; shift
  local image="mysql-80"

  mkdir -p "${path}"
  docker build "${PROGDIR}/docker" -t "${image}"

  docker run -p "${port}:3306" \
    -v "${path}:/var/lib/mysql" \
    -v "scripts:/docker-entrypoint-initdb.d/" \
    --name "${name}" \
    -e "MYSQL_ROOT_PASSWORD=root" \
    -d "${image}" \
    --server-id=1 \
    --log-bin=/var/lib/mysql/binlog.index \
    --binlog-format=row \
    --default-authentication-plugin=mysql_native_password
}

reset() {
  # Use first arg for mysql volume mount, or ~/mysql-cdc as default
  local mysql_volume_mount=${1:-"${HOME}/mysql-cdc"}; shift

  pkill_producer_consumer
  fluvio_uninstall
  docker_kill_mysql
  docker_rm_mysql
  docker_rmi_mysql
  clear_mysql_volume "${mysql_volume_mount}"
}

# shellcheck disable=SC2120
setup() {
  # Delay is 30 seconds by default, or the value of the first arg
  local delay="${1:-30}"; shift

  fluvio_install
  sleep 1
  fluvio_topic

  docker_mysql "mysql-producer" "${HOME}/mysql-cdc/mysql-producer" "3080"
  docker_mysql "mysql-consumer" "${HOME}/mysql-cdc/mysql-consumer" "3090"
  echo "Sleeping ${delay} until MySQL containers are up"
  sleep "${delay}"
}

# shellcheck disable=SC2120
run_producer() {
  local profile="${1:-"${PROGDIR}/producer_profile.toml"}"; shift
  local output="${1:-"/tmp/producer.txt"}"; shift
  cargo run --bin cdc-producer -- "${profile}" > "${output}" 2>&1 &
}

# shellcheck disable=SC2120
run_consumer() {
  local profile="${1:-"${PROGDIR}/consumer_profile.toml"}"; shift
  local output="${1:-"/tmp/consumer.txt"}"; shift
  RUST_LOG=debug cargo run --bin cdc-consumer -- "${profile}" | tee "${output}" &
}

# shellcheck disable=SC2120
test_mysql() {
  # Take sql script from first arg, or use default
  local sql_script="${1:-"${PROGDIR}/test_files/producer_script.sql"}"; shift

  mysql -h 0.0.0.0 -P 3080 -ufluvio -pfluvio4cdc! -v < "${sql_script}"
}

usage() {
  echo "setup.sh [reset] [test]"
  echo "  reset: will uninstall and reinstall local fluvio and mysql"
  echo "   test: will run mysql producer and consumer tests"
}

main() {
  local mysql_volume_mount="${HOME}/mysql-cdc"

  if [[ $1 == "help" ]]; then
    usage
    exit
  fi

  if [[ $1 == "reset" ]]; then
    shift
    reset "${mysql_volume_mount}"
  fi

  setup
  run_producer
  run_consumer

  if [[ $1 == "test" ]]; then
    shift
    test_mysql
  fi

  return
}

main "$@"
