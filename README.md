# LLM

Kinode process for interacting with LLMs.

- [LLM](#llm)
  - [Local LLMs](#local-llms)
    - [Running Local LLMs with Messages](#running-local-llms-with-messages)
    - [Running Local LLMS with Test Scripts](#running-local-llms-with-test-scripts)
  - [Online APIs](#online-apis)
    - [Calling APIs Through Messages](#calling-apis-through-messages)
    - [Calling APIs Through Test Scripts](#calling-apis-through-test-scripts)

## Local LLMs

To run the lccp component, follow these steps:

***Terminal 1:***  Download `llama.cpp` from the GitHub repository: <https://github.com/ggerganov/llama.cpp>

   ```bash
   cd llama.cpp
   make
   ./server -m ../llama.cpp-sharding.cpp/phi2.gguf --embedding --port 3000
   ```

***Terminal 2*** Start a fake node by running:

   ```bash
   kit f
   ```

***Terminal 3*** Build and start the lccp service:

   ```bash
   kit bs lccp/
   ```

### Running Local LLMs with Messages

TODO

### Running Local LLMS with Test Scripts

Run the tester script in your fakenode:

   ```bash
   lccp_tester::llm:kinode
   ```

Within the tester, you can see how different requests and responses are handled.  

## Online APIs

***Terminal 1*** Start a fake node by running:

   ```bash
   kit f
   ```

***Terminal 2*** Build and start the openai service:

   ```bash
   kit bs openai/
   ```

### Calling APIs Through Messages

TODO

### Calling APIs Through Test Scripts

Run the tester script in your fakenode:

***Terminal 1*** Run the tester script

   ```bash
   openai_tester::llm:kinode
   ```

Within the tester, you can see how different requests and responses are handled.
