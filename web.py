from pathlib import Path
from llama_index import GPTSimpleVectorIndex
import flask


def run(index_file: Path):
    model = GPTSimpleVectorIndex.load_from_disk(str(index_file))
    flask_app = flask.Flask(__name__)

    def homepage():
        return flask.render_template("index.html")

    def query_api():
        query_str: str = flask.request.json["search"]  # type: ignore
        if not query_str.strip():
            return "Empty strings are not accepted", 400
        result = model.query(query_str)
        return flask.jsonify(response=result)
    flask_app.route("/", methods=["GET"])(homepage)
    flask_app.route("/query", methods=["POST", "GET"])(query_api)
    flask_app.run(host="0.0.0.0", port="4100")
