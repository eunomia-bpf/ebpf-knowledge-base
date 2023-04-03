import os
import sys
import pathlib
import shutil
import train
import extract
ENV_VAR_NAME = "OPENAI_API_KEY"
LOCK_FILE_LLAMA = "train_llama.lock"
LOCK_FILE_LANGCHAIN = "train_langchain.lock"
LOCAL_PATH = pathlib.Path(__file__).resolve().parent

sys.executable


def main() -> int:
    if ENV_VAR_NAME not in os.environ:
        print("FATAL: Set environment variable OPENAI_API_KEY to your api key. Get that from https://platform.openai.com/account/api-keys", file=sys.stderr)
        return 1

    if not os.path.exists(LOCAL_PATH/LOCK_FILE_LLAMA) or not os.path.exists(LOCAL_PATH/LOCK_FILE_LANGCHAIN):
        print(
            f"{LOCK_FILE_LLAMA} or {LOCK_FILE_LANGCHAIN} not found.")
        out_dir = LOCAL_PATH/"data"
        os.makedirs(out_dir/"docs", exist_ok=True)
        if os.path.exists(out_dir/"docs"):
            shutil.rmtree(out_dir/"docs")
            os.makedirs(out_dir/"docs")
        extract.extract_docs(LOCAL_PATH/"tutorial-docs", out_dir/"docs")
        if not os.path.exists(LOCAL_PATH/LOCK_FILE_LLAMA):
            print("Training for llama_index..")
            train.train(out_dir/"docs", out_dir)
            with open(LOCAL_PATH/LOCK_FILE_LLAMA, "w") as f:
                f.write("")
        if not os.path.exists(LOCAL_PATH/LOCK_FILE_LANGCHAIN):
            print("Training for langchain..")
            train.train_langchain(out_dir/"docs", out_dir)
            with open(LOCAL_PATH/LOCK_FILE_LANGCHAIN, "w") as f:
                f.write("")

    else:
        print(f"{LOCK_FILE_LLAMA} and {LOCK_FILE_LANGCHAIN} found. Skipped training process. If you want to retrain, just delete {LOCAL_PATH/LOCK_FILE_LLAMA} or {LOCAL_PATH/LOCK_FILE_LANGCHAIN}")
    print("Starting web server..")
    os.execvp("./web-server-executable", ["web-server-executable"])


if __name__ == "__main__":
    exit(main())
