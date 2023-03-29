import os
import sys
import pathlib
import shutil
import web
import train
import extract
ENV_VAR_NAME = "OPENAI_API_KEY"
LOCK_FILE = "train.lock"

LOCAL_PATH = pathlib.Path(__file__).resolve().parent


def main() -> int:
    if ENV_VAR_NAME not in os.environ:
        print("FATAL: Set environment variable OPENAI_API_KEY to your api key. Get that from https://platform.openai.com/account/api-keys", file=sys.stderr)
        return 1
    if not os.path.exists(LOCAL_PATH/LOCK_FILE):
        print(f"{LOCK_FILE} not found. Training..")
        out_dir = LOCAL_PATH/"data"
        if os.path.exists(out_dir):
            shutil.rmtree(out_dir, ignore_errors=True)
        os.makedirs(out_dir/"docs")
        extract.extract_docs(LOCAL_PATH/"tutorial-docs", out_dir/"docs")
        train.train(out_dir/"docs", out_dir)
        with open(LOCAL_PATH/LOCK_FILE, "w") as f:
            f.write("")
    else:
        print(f"{LOCK_FILE} found. Skipped training process. If you want to retrain, just delete {LOCAL_PATH/LOCK_FILE}")
    print("Starting web server..")
    web.run(LOCAL_PATH/"data"/"train_data.json")
    return 0


if __name__ == "__main__":
    exit(main())
