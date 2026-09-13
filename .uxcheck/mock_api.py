import json
from http.server import BaseHTTPRequestHandler, HTTPServer

CHUNKS = [
    "---\ngenre: Mock\n---\n\n# Genre\n\n",
    "Premise: the mock premise for the walkthrough.\n",
    "Central question: does the desktop shell hold up?\n",
    "## tone_notes\n\nMock tone notes long enough to pass the preview check.\n",
]


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = json.dumps({"data": [{"id": "mock-large"}]}).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        self.rfile.read(int(self.headers.get("content-length", 0)))
        self.send_response(200)
        self.send_header("content-type", "text/event-stream")
        self.end_headers()
        for piece in CHUNKS:
            self.wfile.write(f"data: {json.dumps({'choices': [{'delta': {'content': piece}}]})}\n\n".encode())
            self.wfile.flush()
        self.wfile.write(b"data: [DONE]\n\n")

    def log_message(self, *args):
        pass


HTTPServer(("127.0.0.1", 8096), Handler).serve_forever()
