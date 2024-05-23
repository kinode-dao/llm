# LLM

Kinode processes for interacting with & serving LLMs.

## Usage

### Get llamafile & your favorite LLM in GGUF format.

```
# Get llamafile.
export LLAMAFILE_PATH=$(realpath ~/llms)
mkdir -p $LLAMAFILE_PATH
cd $LLAMAFILE_PATH

export LLAMAFILE_VERSION=0.8.4
curl -L -o llamafile-${LLAMAFILE_VERSION} https://github.com/Mozilla-Ocho/llamafile/releases/download/${LLAMAFILE_VERSION}/llamafile-${LLAMAFILE_VERSION}
chmod +x llamafile-${LLAMAFILE_VERSION}

# Get your favorite LLM in GGUF format.
# E.g. Llama-3-8B-Instruct
cd $LLAMAFILE_PATH
curl -L -o Meta-Llama-3-8B-Instruct-Q8_0.gguf https://huggingface.co/lmstudio-community/Meta-Llama-3-8B-Instruct-GGUF/resolve/main/Meta-Llama-3-8B-Instruct-Q8_0.gguf?download=true

# Serve the LLM with llamafile.
# Note the port; below we assume it is 8080.
cd $LLAMAFILE_PATH
./llamafile-${LLAMAFILE_VERSION} -m Meta-Llama-3-8B-Instruct-Q8_0.gguf --server --mlock --nobrowser
export LLAMAFILE_PORT=8080
```

### Get kit & Kinode core develop (requires 0.8.0).

```
# Get kit.
cargo install --git https://github.com/kinode-dao/kit --branch develop

# Get Kinode core develop.
export KINODE_PATH=$(realpath ~/kinode)
cd $KINODE_PATH

git clone git@github.com:kinode-dao/kinode.git
cd kinode
git checkout develop

# Get llm_provider.
cd $KINODE_PATH
git clone git@github.com:kinode-dao/llm
cd llm
git checkout hf/llm-provider
```

### Start fakenodes.

```
# Start some fakenodes.
# The first will be the router.
# The second the driver connected to the llamafile server.
# The third the client sending a request -- routed through:
# * third's driver,
# * first's router,
# * second's driver,
# * llamafile,
# * back again.
export ROUTER_PORT=8081
export DRIVER_PORT=8082
export CLIENT_PORT=8083
kit f --runtime-path ${KINODE_PATH}/kinode --port $ROUTER_PORT

# In a new terminal:
kit f -r ${KINODE_PATH}/kinode -h /tmp/kinode-fake-node-2 -p $DRIVER_PORT -f fake2.dev

# In a third terminal:
kit f -r ${KINODE_PATH}/kinode -h /tmp/kinode-fake-node-3 -p $CLIENT_PORT -f fake3.dev
```

### Build and install llm provider.

```
cd ${KINODE_PATH}/llm
kit b
kit s $ROUTER_PORT && kit s $DRIVER_PORT && kit s $CLIENT_PORT
```

### Configure routers, drivers, clients.

```
# Start a router (router node terminal).
admin:llm_provider:nick1udwig.os "StartRouter"

# Set driver to use router (driver node terminal).
admin:llm_provider:nick1udwig.os {"SetRouter": {"node": "fake.dev"}}
admin:llm_provider:nick1udwig.os {"SetLocalDriver": {"model": "llama3-8b", "is_public": true}}

# Send jobs from inside Kinode (client node terminal).
admin:llm_provider:nick1udwig.os {"SetRouter": {"node": "fake.dev"}}
run_job:llm_provider:nick1udwig.os llama3 How much wood could a woodchuck chuck? Be concise.

# Send jobs from outside Kinode (unix terminal).
kit i driver:llm_provider:nick1udwig.os '{"RunJob": {"model": "llama3-8b", "prompt": "What is the funniest book in the Bible? Be concise."}}' -p $CLIENT_PORT
```

## Architecture

