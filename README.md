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
```

### Get kit & Kinode core develop (requires 0.8.0).

```
# Get kit.
cargo install --git https://github.com/kinode-dao/kit --branch develop

# Get Kinode core develop.
export KINODE_PATH=$(realpath ~/kinode)
mkdir -p $KINODE_PATH
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
export KINODE_PATH=$(realpath ~/kinode)
export ROUTER_PORT=8081
export DRIVER_PORT=8082
export CLIENT_PORT=8083
# The first fake node to be booted will compile Kinode core, which will take some time.
kit f --runtime-path ${KINODE_PATH}/kinode --port $ROUTER_PORT

# In a new terminal:
export KINODE_PATH=$(realpath ~/kinode)
export ROUTER_PORT=8081
export DRIVER_PORT=8082
export CLIENT_PORT=8083
kit f -r ${KINODE_PATH}/kinode -h /tmp/kinode-fake-node-2 -p $DRIVER_PORT -f fake2.dev

# In a third terminal:
export KINODE_PATH=$(realpath ~/kinode)
export ROUTER_PORT=8081
export DRIVER_PORT=8082
export CLIENT_PORT=8083
kit f -r ${KINODE_PATH}/kinode -h /tmp/kinode-fake-node-3 -p $CLIENT_PORT -f fake3.dev
```

### Build and install llm provider.

```
cd ${KINODE_PATH}/llm
kit b
kit s -p $ROUTER_PORT && kit s -p $DRIVER_PORT && kit s -p $CLIENT_PORT
```

### Configure routers, drivers, clients.

```
# Start a router (router node terminal).
admin:llm_provider:nick1udwig.os "StartRouter"

# Set driver to use router (driver node terminal).
# All drivers must point to the router: both clients and providers.
admin:llm_provider:nick1udwig.os {"SetRouter": {"node": "fake.dev"}}

# Set an OpenAI API provider (driver node terminal).
# NOTE: Replace the `8080` port here with $LLAMAFILE_PORT
m our@openai:llm_provider:nick1udwig.os '{"RegisterOaiProviderEndpoint": {"endpoint": "http://127.0.0.1:8080/v1"}}'

# Set driver to use router (driver node terminal).
admin:llm_provider:nick1udwig.os {"SetLocalDriver": {"model": "llama3-8b", "is_public": true}}

# Send jobs from inside Kinode (client node terminal).
admin:llm_provider:nick1udwig.os {"SetRouter": {"node": "fake.dev"}}
run_job:llm_provider:nick1udwig.os llama3 How much wood could a woodchuck chuck? Be concise.
```

## Architecture

Clients send Requests to the `driver:llm_provider:nick1udwig.os` process.
If connected to an LLM, e.g. via
```
m our@openai:llm_provider:nick1udwig.os '{"RegisterOaiProviderEndpoint": {"endpoint": "http://127.0.0.1:8080/v1"}}'
admin:llm_provider:nick1udwig.os {"SetLocalDriver": {"model": "llama3-8b", "is_public": true}}
```
`driver` will serve the Request locally by passing it to `openai:llm_provider:nick1udwig.os`, which in turn passes it to the registered OpenAI API provider (here, a llamafile server).

![240523-local](https://github.com/kinode-dao/llm/assets/79381743/47b1c23c-03db-4076-a61f-5e037c66d848)

If not connected to an LLM, `driver` will contact the `router_node` it has been set to coordinate with and that `router` will forward the Request to a `driver` that has registered as serving the appropriate model.

![240523-remote](https://github.com/kinode-dao/llm/assets/79381743/3f87f855-81db-4c08-ad84-a63827c23b16)

## Future work / extensions

### On-chain coordination, payment, etc.

There are multiple ways on-chain compute & data could be used here.
A few examples are:
1. Payment for services rendered,
2. Coordination of router(s) and drivers to improve quality of service by kicking out bad actors,
3. Gating based on on-chain data.

An example of 2 can be seen in the following repos:
* https://github.com/nick1udwig/provider-dao-contract
* https://github.com/nick1udwig/provider_dao_router
* https://github.com/nick1udwig/comfyui_provider
* https://github.com/nick1udwig/comfyui_client

### Gate based on arbitrary conditions

The `router` is the natural place to gate requests based on arbitrary conditions.
Upon receiving a `ClientRequest::RunJob`, execute arbitrary logic to determine if the requestor should be served or not.
This logic might look like:
1. Comparison with an in-memory `Vec` of allowed nodes,
2. Checking if the requestor holds a certain NFT,
3. Checking if the requestor has paid into an escrow contract,
to list a few examples.

### Chunked/non-final `JobUpdate`s

Currently, one `JobUpdate` is sent as a result of a request.
That is the final output of the LLM.
Instead, `JobUpdate`s could be sent as they arrive and a job would only be considered complete once `is_final = true`.
This gets around a limitation of the current system which is that the request must be completed within 60s.

### Queue for driver

The `driver` currently only allows one outstanding job at a time.

### Discriminate based on model

Currently the code the discriminate based on model (i.e. only pass requests specifying a given model to providers serving that specific model) is commented out.
