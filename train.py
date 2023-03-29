import os
from pathlib import Path
from llama_index import GPTSimpleVectorIndex, SimpleDirectoryReader


def train(doc_path: Path, out_dir: Path):

    documents = SimpleDirectoryReader(str(doc_path)).load_data()
    index = GPTSimpleVectorIndex(documents)
    index.save_to_disk(str(out_dir/"train_data.json"))
