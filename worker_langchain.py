import json
import pickle
import sys
import faiss
from langchain.chat_models import ChatOpenAI
# from langchain import OpenAI
from langchain.chains import VectorDBQAWithSourcesChain
from langchain.vectorstores import FAISS
from langchain.chains.question_answering import load_qa_chain
import pathlib
LOCAL_PATH = pathlib.Path(__file__).resolve().parent


def main():
    try:
        index = faiss.read_index(str(LOCAL_PATH/"data"/"langchain_docs.index"))
        with open(LOCAL_PATH/"data/"/"langchain_store.pkl", "rb") as f:
            store: FAISS = pickle.load(f)
        store.index = index
        llm = ChatOpenAI(temperature=0.9)  # type: ignore
        # chain = VectorDBQAWithSourcesChain.from_llm(llm=llm, vectorstore=store)
        chain = load_qa_chain(llm, "stuff")
    except Exception as ex:
        print(f"Failed: {str(ex)}")
        exit(1)
    print("done")
    while True:
        try:
            s = json.loads(input())
        except EOFError as _:
            break
        except KeyboardInterrupt as _:
            break
        task = s["task"]
        if task == "exit":
            break
        elif task == "query":
            try:
                query = s["query"]
                docs = store.similarity_search(query)
                ret = chain.run(input_documents=docs, question=query)
                # ret = chain({"question": query})["answer"]
            except Exception as ex:
                print(json.dumps({"reply": str(ex), "ok": False}))
                import traceback
                traceback.print_exc(file=sys.stderr)
            else:
                print(json.dumps({"reply": str(ret), "ok": True}))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
