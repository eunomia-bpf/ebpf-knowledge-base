import os
import pickle
import faiss
from pathlib import Path
from llama_index import GPTSimpleVectorIndex, SimpleDirectoryReader
from langchain.vectorstores import FAISS
from langchain.embeddings import OpenAIEmbeddings
from langchain.document_loaders.directory import DirectoryLoader
from langchain.docstore.document import Document
from langchain.document_loaders.base import BaseLoader
from langchain.text_splitter import CharacterTextSplitter
from langchain.embeddings import OpenAIEmbeddings
from typing import List


def train(doc_path: Path, out_dir: Path):
    documents = SimpleDirectoryReader(str(doc_path)).load_data()
    index = GPTSimpleVectorIndex(documents)
    index.save_to_disk(str(out_dir/"train_data_llama.json"))


def train_langchain(doc_path: Path, out_dir: Path):
    doc_files = [doc_path/s for s in os.listdir(doc_path)]
    data = []
    sources = []
    for p in doc_files:
        with open(p, "r", encoding="utf-8") as f:
            data.append(f.read())
        sources.append(p)
    text_splitter = CharacterTextSplitter(chunk_size=1500, separator="\n")
    docs = []
    metadatas = []
    for i, d in enumerate(data):
        splits = text_splitter.split_text(d)
        docs.extend(splits)
        metadatas.extend([{"source": sources[i]}] * len(splits))
    store = FAISS.from_texts(docs, OpenAIEmbeddings(), metadatas=metadatas)
    faiss.write_index(store.index, str(out_dir/"langchain_docs.index"))
    store.index = None
    with open(out_dir/"langchain_store.pkl", "wb") as f:
        pickle.dump(store, f)
