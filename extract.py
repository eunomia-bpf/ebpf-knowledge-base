import os
import shutil
import pathlib

OUT_DIR = pathlib.Path("code/data_import/source_data")

DROP_FILES = {
    "kernel-versions.md",
    "reference_guide.md"
}


def copy_file(out_dir: pathlib.Path, src: pathlib.Path, dst_name: str):
    shutil.copy(src, out_dir/f"{dst_name}.txt")


def extract_docs(tutorial_root: pathlib.Path, out_dir: pathlib.Path):
    doc_path = tutorial_root / "src"

    copy_file(out_dir, tutorial_root/"README.md", "main")
    copy_file(out_dir, doc_path/"SUMMARY.md", "summary")
    for dir in (t for t in os.listdir(doc_path) if os.path.isdir(doc_path/t) and not t.startswith("bcc")):
        copy_file(out_dir, doc_path/dir/"README.md", dir)
    for doc in os.listdir(doc_path/"bcc-documents"):
        if doc not in DROP_FILES:
            copy_file(out_dir, doc_path / "bcc-documents"/doc, doc)
