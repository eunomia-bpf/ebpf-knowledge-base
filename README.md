# ebpf-knowledge-base

An ebpf knowledge base, based on [llama_index](https://github.com/jerryjliu/llama_index) and [bpf-developer-tutorial](https://github.com/eunomia-bpf/bpf-developer-tutorial)

## Usage

First, you need to clone this repo:

`git clone --recursive https://github.com/eunomia-bpf/ebpf-knowledge-base`

Then, there are two ways to use thie it:

1. Run `docker build -t ebpf-knowledge-base .` to build the docker image, and use `docker run -e OPENAPI_API_KEY=<YOU KEY HERE> -p 4100:4100 ebpf-knowledge-base` to run it. It will serve a web pages on port `4100`.
2. Run `pip install -r requirements.txt`, and run `entry.py`
