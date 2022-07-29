export CARGO_HOME="$(pwd)/cargo-home"
export CARGO_TARGET_DIR="$(pwd)/cargo-target-dir"

function unpack_deps() {
  if [[ -f ${REPO_PATH}/yarn.lock ]]; then
    echo "Unpacking nodejs deps... "

    pushd ${REPO_PATH:-repo} > /dev/null

    tar -zxvf ../bundled-deps/bundled-deps-*.tgz ./node_modules/ ./yarn.lock > /dev/null

    if [[ "$(git status -s -uno)" != "" ]]; then
      echo "Extracting deps has created a diff - deps are not in sync"
      git --no-pager diff
      exit 1;
    fi

    echo "Done!"

    popd
  else
    echo "Copying from ${CARGO_TARGET_DIR}"
    cargo build
    cp -r ${CARGO_TARGET_DIR} ${REPO_PATH}/target
  fi
}
