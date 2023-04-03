import json
import sys
from llama_index import GPTSimpleVectorIndex
import pathlib
LOCAL_PATH = pathlib.Path(__file__).resolve().parent


def main():
    try:
        model = GPTSimpleVectorIndex.load_from_disk(str(LOCAL_PATH / "data" / "train_data_llama.json"))
    except KeyboardInterrupt as ex:
        print(f"Failed to load model: {ex}", file=sys.stderr)
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
                ret = model.query(s["query"])
            except Exception as ex:
                print(json.dumps({"reply":str(ex), "ok":False}))
                import traceback
                traceback.print_exc(file=sys.stderr)
            else:
                print(json.dumps({"reply": str(ret), "ok": True}))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
